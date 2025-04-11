use crate::{state::AppState, templates::TEMPLATES, util::optimize_and_save_model};
use axum::{
    Form, Json, Router,
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post, put},
};
use axum_extra::extract::{CookieJar, Multipart, cookie::Cookie};
use entity::{card_design, prelude::*, user};
use sea_orm::{
    ActiveValue::Set, EntityTrait, IntoActiveModel, IntoIdentity, QueryFilter, prelude::Expr,
};
use std::path::PathBuf;
use tera::Context;

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
        .route("/account", get(account))
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
    let scans = Scan::find()
        .filter(Expr::col(entity::scan::Column::Username).eq(user.username.clone()))
        .all(&*state.db)
        .await
        .unwrap();

    let mut cards = Vec::new();

    for scan in scans {
        let card = Card::find_by_id(&scan.card)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        let design = CardDesign::find_by_id(card.design)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        cards.push(CardData {
            design_id: design.id.to_string(),
            card_id: Some(scan.card),
            team_number: design.team as u64,
            team_name: CardDesign::find_by_id(design.team)
                .one(&*state.db)
                .await
                .unwrap()
                .unwrap()
                .name,
            bot_name: Some(design.name),
            year: design.year as u16,
            abilities: None,
        });
    }

    context.insert("is_auth", &true);
    context.insert("user_name", &user.username);
    context.insert("profile_pic", "/images/default-profile.png");
    context.insert("cards_collected", &cards);
    if let Some(team_num) = user.team {
        let team = Team::find_by_id(team_num)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
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
    if !user.is_admin {
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

async fn account(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);
    context.insert("username", "robotics_fan123");

    let has_team = true;
    context.insert("has_team", &has_team);

    if has_team {
        context.insert("team_name", "Phoenix");
        context.insert("team_number", "4533");

        let is_team_admin = true;
        context.insert("is_team_admin", &is_team_admin);
    }

    let content = TEMPLATES.render("account.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
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
    let user = User::find_by_id(&form.username)
        .one(&*state.db)
        .await
        .unwrap()
        .unwrap();
    if !state.check_user_password(&user.password, &form.password) {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Invalid username or password".to_string())
            .unwrap()
            .into_response()
    } else {
        let mut cookie = Cookie::new("token", state.register_token(&user.username).await);
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
    if user.is_admin {
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
            let token = AuthToken::find_by_id(tok.value()).one(&*state.db).await;
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

pub struct Auth(pub user::Model);
impl FromRequestParts<AppState> for Auth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut jar = CookieJar::from_headers(&parts.headers);
        if let Some(tok) = jar.get("token") {
            let token = AuthToken::find_by_id(tok.value())
                .one(&*state.db)
                .await
                .unwrap();
            if token.is_some() {
                return Ok(Auth(
                    User::find_by_id(token.unwrap().user)
                        .one(&*state.db)
                        .await
                        .unwrap()
                        .unwrap()
                        .to_owned(),
                ));
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
    let designs = CardDesign::find().all(&*state.db).await.unwrap();

    let mut cards = Vec::new();

    for design in designs {
        cards.push(CardData {
            design_id: design.id.to_string(),
            card_id: None,
            team_number: design.team as u64,
            team_name: Team::find_by_id(design.team)
                .one(&*state.db)
                .await
                .unwrap()
                .unwrap()
                .name,
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
    if let Some(team_num) = user.team {
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

        CardDesign::insert(card_design::ActiveModel {
            team: Set(team_num),
            name: Set(bot_name.unwrap()),
            year: Set(2025),
            ..Default::default()
        })
        .exec(&*state.db)
        .await
        .unwrap();
    }
}

async fn scan_card(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let card = Card::find_by_id(&id)
        .one(&*state.db)
        .await
        .unwrap()
        .unwrap();
    let design = CardDesign::find_by_id(card.design)
        .one(&*state.db)
        .await
        .unwrap()
        .unwrap();
    let team = if let Some(team_num) = user.team {
        Team::find_by_id(team_num).one(&*state.db).await.unwrap()
    } else {
        None
    };

    Json(CardData {
        bot_name: Some(design.name),
        card_id: Some(id),
        design_id: design.id.to_string(),
        team_number: team.clone().map(|x| x.number).unwrap_or_default() as u64,
        team_name: team.map(|x| x.name).unwrap_or_default().clone(),
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

async fn get_user(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(username): Path<String>,
) -> impl IntoResponse {
    dbg!(&username);
    if user.is_admin {
        let user = User::find_by_id(username)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        Json(UserData {
            username: user.username.clone(),
            is_admin: user.is_admin,
            is_verified: user.is_verified,
            team: None, //user.team().get(&state.db).await.unwrap().map(|t| t.name.clone()),
        })
        .into_response()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap()
            .into_response()
    }
}

async fn get_users(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    if user.is_admin {
        let users = User::find().all(&*state.db).await.unwrap();
        Json(
            users
                .iter()
                .map(|u| UserData {
                    username: u.username.clone(),
                    is_admin: u.is_admin,
                    is_verified: u.is_verified,
                    team: None, //u.team
                })
                .collect::<Vec<_>>(),
        )
        .into_response()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap()
            .into_response()
    }
}

async fn modify_user(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(username): Path<String>,
    Json(data): Json<UserData>,
) -> impl IntoResponse {
    if user.is_admin {
        let mut user = User::find_by_id(username)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        user.is_admin = Set(data.is_admin);
        user.is_verified = Set(data.is_verified);
        User::update(user).exec(&*state.db).await.unwrap();
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
