use crate::{
    db::{AuthToken, Card, CardAbility, CardDesign, User},
    state::AppState,
    templates::TEMPLATES,
    util::optimize_and_save_model,
};
use axum::{
    body::Body, extract::{FromRequestParts, Path, Query, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Redirect, Response}, routing::{get, post, put}, Form, Json, Router
};
use axum_extra::extract::{cookie::Cookie, CookieJar, Multipart};
use std::path::PathBuf;
use tera::Context;
use toasty::stmt::Select;

use tower_http::services::ServeDir;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(hero))
        .route("/about", get(about))
        .route("/privacy", get(privacy))
        .route("/signin", get(signin))
        .route("/signup", get(signup))
        .route("/cards", get(cards))
        .route("/scan", get(scan))
        .route("/dashboard", get(dashboard))
        .route("/admin", get(admin))
        .with_state(state.clone())
        .nest("/api", get_api_router(state))
        .fallback_service(ServeDir::new(PathBuf::from("public")))
}

pub fn get_api_router(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/register", post(register))
        .route("/cards", get(get_cards).post(create_card))
        .route("/scan", post(scan_card))
        .route("/user/{username}", get(get_user).put(modify_user))
        .route("/users", get(get_users))
        //.route("/cards/:id", get(get_card).put(update_card).delete(delete_card))
        //.route("/collection", get(get_collection).put(add_to_collection))
        .route("/invites", put(create_invite_code))
        .with_state(state)
}

async fn scan(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("scan.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn hero(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("hero.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn about(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("about.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn privacy(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("privacy.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn signin(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("signin.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn signup(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("signup.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn cards(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    let content = TEMPLATES.render("cards.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn dashboard(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    // In a real application, we would fetch this data from a database
    // based on the authenticated user's session
    let mut context = Context::new();
    let scans = user
        .scans()
        .all(&state.db)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .unwrap();

    let mut cards = Vec::new();

    for scan in scans {
        let card = Card::get_by_id(&state.db, &scan.card_id).await.unwrap();
        let design = card.card_design().get(&state.db).await.unwrap();
        cards.push(CardData {
            design_id: design.id.clone(),
            card_id: Some(scan.card_id),
            team_number: design.team_number as u64,
            team_name: design.team().get(&state.db).await.unwrap().name,
            bot_name: Some(design.name),
            year: design.year as u16,
            abilities: None,
        });
    }

    context.insert("is_auth", &true);
    context.insert("user_name", &user.username);
    context.insert("profile_pic", "/images/default-profile.png");
    context.insert("cards_collected", &cards);
    if let Some(team) = user.team().get(&state.db).await.unwrap() {
        context.insert("has_team", &true);
        context.insert("team_name", &team.name);
        context.insert("team_number", &(team.number as u64));
    } else {
        context.insert("has_team", &false);
    }

    let content = TEMPLATES.render("dashboard.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn admin(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    // Check if the user is a site admin
    if user.is_admin.is_none() {
        return Redirect::to("/dashboard").into_response();
    }

    let mut context = Context::new();
    context.insert("is_auth", &true);
    context.insert("user_name", &user.username);

    let content = TEMPLATES.render("admin.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap()
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    mut jar: CookieJar,
    form: Form<LoginForm>,
) -> impl IntoResponse {
    let user = User::get_by_username(&state.db, &form.username)
        .await
        .unwrap();
    if !state.check_user_password(&user.password, &form.password) {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Invalid username or password".to_string())
            .unwrap()
            .into_response()
    } else {
        let mut cookie = Cookie::new("token", state.register_token(&user).await);
        cookie.set_path("/");
        jar = jar.add(cookie);

        (jar, Redirect::to("/").into_response()).into_response()
    }
}

async fn logout(State(state): State<AppState>, mut jar: CookieJar) -> impl IntoResponse {
    jar = jar.remove(Cookie::from("token"));
    (jar, Redirect::to("/").into_response()).into_response()
}

async fn register(State(state): State<AppState>, form: Form<LoginForm>) -> impl IntoResponse {
    state
        .create_user(None, &form.username, &form.password)
        .await;

    Response::builder()
        .status(StatusCode::OK)
        .body("Registration successful".to_string())
        .unwrap()
}

async fn create_invite_code(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    if user.is_admin.is_none() {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not allowed to create invite codes".to_string())
            .unwrap()
    } else {
        let invite_code = state.create_invite_code(&user.username).await.unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(invite_code.to_string())
            .unwrap()
    }
}

pub struct IsAuth(pub bool);
impl FromRequestParts<AppState> for IsAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut jar = CookieJar::from_headers(&parts.headers);
        if let Some(tok) = jar.get("token") {
            let token = AuthToken::get_by_token(&state.db, tok.value()).await;
            if token.is_ok() {
                return Ok(IsAuth(true));
            } else {
                jar = jar.remove(Cookie::from("token"));
                Err((jar, Redirect::to("/login").into_response()).into_response())
            }
        } else {
            Ok(IsAuth(false))
        }
    }
}

pub struct Auth(pub User);
impl FromRequestParts<AppState> for Auth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut jar = CookieJar::from_headers(&parts.headers);
        if let Some(tok) = jar.get("token") {
            let token = AuthToken::get_by_token(&state.db, tok.value()).await;
            if token.is_ok() {
                return Ok(Auth(token.unwrap().user().get(&state.db).await.unwrap()));
            } else {
                jar = jar.remove(Cookie::from("token"));
                Err((jar, Redirect::to("/login").into_response()).into_response())
            }
        } else {
            Err(Redirect::to("/login").into_response())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CardData {
    design_id: String,
    card_id: Option<String>,
    team_number: u64,
    team_name: String,
    year: u16,
    bot_name: Option<String>,
    abilities: Option<Vec<CardAbilityData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CardAbilityData {
    stat: u8,
    title: String,
    description: String,
}

#[derive(Deserialize)]
struct GetCardsParams {
    user: Option<String>,
}

//async fn get_cards(State(state): State<AppState>, Query(params): Query<GetCardsParams>) -> impl IntoResponse {
async fn get_cards(State(state): State<AppState>) -> impl IntoResponse {
    let designs = state
        .db
        .all(Select::<CardDesign>::all())
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .unwrap();

    let mut cards = Vec::new();

    for design in designs {
        cards.push(CardData {
            design_id: design.id.clone(),
            card_id: None,
            team_number: design.team_number as u64,
            team_name: design.team().get(&state.db).await.unwrap().name,
            year: design.year as u16,
            bot_name: Some(design.name.clone()),
            abilities: None,
        });
    }

    Json(cards)
}

async fn create_card(
    State(state): State<AppState>,
    Auth(user): Auth,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(team_num) = user.team_number {
        let id = nanoid!(33);

        let mut bot_name = None;
        let mut photo = None;
        let mut model = None;

        while let Some(field) = multipart.next_field().await.unwrap() {
            match field.name() {
                Some("bot_name") => {
                    bot_name = field.text().await.ok();
                }
                Some("photo") => {
                    photo = field.bytes().await.ok();
                }
                Some("model") => {
                    model = field.bytes().await.ok();
                }
                Some(_) | None => {}
            }
        }

        optimize_and_save_model(id.clone(), model.unwrap().to_vec());

        user.team()
            .get(&state.db)
            .await
            .unwrap()
            .unwrap()
            .designs()
            .create()
            .id(id)
            .team_number(team_num)
            .name(bot_name.unwrap())
            .year(2025)
            .exec(&state.db)
            .await
            .unwrap();
    }
}

async fn scan_card(State(state): State<AppState>, Auth(user): Auth, Path(id): Path<String>) -> impl IntoResponse {
    let card = Card::get_by_id(&state.db, &id).await.unwrap();
    let design = card.card_design().get(&state.db).await.unwrap();
    let team = user.team().get(&state.db).await.unwrap().unwrap();

    Json(CardData {
        bot_name: Some(design.name.clone()),
        card_id: Some(id),
        design_id: design.id.clone(),
        team_number: team.number as u64,
        team_name: team.name.clone(),
        year: design.year as u16,
        abilities: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    username: String,
    is_admin: bool,
    is_verified: bool,
    team: Option<String>,
}

async fn get_user(State(state): State<AppState>, Auth(user): Auth, Path(username): Path<String>) -> impl IntoResponse {
    dbg!(&username);
    if user.is_admin.is_some() {
        let user = User::get_by_username(&state.db, username).await.unwrap();
        Json(UserData {
            username: user.username.clone(),
            is_admin: user.is_admin.is_some(),
            is_verified: user.is_verified.is_some(),
            team: None, //user.team().get(&state.db).await.unwrap().map(|t| t.name.clone()),
        }).into_response()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap().into_response()
    }
}

async fn get_users(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    if user.is_admin.is_some() {
        let users = state.db.all(Select::<User>::all()).await.unwrap().collect::<Vec<_>>().await.unwrap();
        Json(users.iter().map(|u| UserData {
            username: u.username.clone(),
            is_admin: u.is_admin.is_some(),
            is_verified: u.is_verified.is_some(),
            team: None, //u.team
        }).collect::<Vec<_>>()).into_response()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap().into_response()
    }
}

async fn modify_user(State(state): State<AppState>, Auth(user): Auth, Path(username): Path<String>, Json(data): Json<UserData>) -> impl IntoResponse {
    if user.is_admin.is_some() {
        let mut user = User::get_by_username(&state.db, username).await.unwrap();
        let mut user = user.update();
        user = user.is_admin(if data.is_admin {
            Some(String::from("skibidi"))
        } else {
            None
        });
        user = user.is_verified(if data.is_verified {
            Some(String::from("skibidi"))
        } else {
            None
        });
        user.exec(&state.db).await.unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body("User updated".to_string())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap()
    }
}
