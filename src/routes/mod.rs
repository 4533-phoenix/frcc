use crate::{state::AppState, templates::TEMPLATES, util::optimize_and_save_model};
use argon2::{
    PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Form, Json, Router,
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, StatusCode, Uri, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post, put},
};
use axum_extra::extract::{CookieJar, Multipart, cookie::Cookie};
use entity::{card_design, prelude::*, user};
use sea_orm::{
    ActiveValue::Set, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder,
    prelude::Expr, sqlx::types::chrono,
};
use std::path::PathBuf;
use tera::Context;

#[derive(RustEmbed)]
#[folder = "public"]
struct Assets;

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
        .fallback_service(get(static_handler))
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    if let Some(asset) = Assets::get(uri.path().trim_start_matches('/')) {
        Response::builder()
            .header(header::CONTENT_TYPE, asset.metadata.mimetype())
            .body(Body::from(asset.data.to_vec()))
            .unwrap()
            .into_response()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found".as_bytes().to_vec()))
            .unwrap()
            .into_response()
    }
}

pub fn get_api_router(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/register", post(register))
        .route("/cards", get(get_cards).post(create_card))
        .route("/scans", put(do_scan).get(get_scans))
        .route("/user/{username}", get(get_user).put(modify_user))
        .route("/users", get(get_users))
        .route("/invites", put(create_invite_code))
        .route("/account/join_team", post(join_team))
        .route("/account/leave_team", post(leave_team))
        .route("/account/change_password", post(change_password))
        .route("/team/{team_num}", get(get_team).post(modify_team))
        .route("/team/{team_num}/members", get(get_team_members))
        .route(
            "/team/{team_num}/member/{username}",
            put(modify_team_member),
        )
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
    let mut context = build_context(Some(user.clone()), state.clone()).await;

    let scans = Scan::find()
        .filter(Expr::col(entity::scan::Column::Username).eq(user.username.clone()))
        .all(&*state.db)
        .await
        .unwrap();

    context.insert("profile_pic", "/images/default-profile.png");
    context.insert("cards_collected", &scans.len());

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
    let context = build_context(Some(user), state).await;

    let content = TEMPLATES.render("admin.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap()
}

async fn build_context(user: Option<entity::user::Model>, state: AppState) -> Context {
    let mut context = Context::new();

    context.insert("is_auth", &user.is_some());

    if let Some(user) = user {
        context.insert("username", &user.username);
        context.insert("is_site_admin", &user.is_admin);

        let team = UserTeam::find_by_id(&user.username)
            .one(&*state.db)
            .await
            .unwrap()
            .map(|ut| ut.team);
        context.insert("has_team", &team.is_some());

        if let Some(team) = team {
            let team = state.get_team(team).await;
            context.insert("team_name", &team.name);
            context.insert("team_number", &team.number);

            context.insert("is_team_admin", &state.is_team_admin(&user.username).await);
        }
    }

    context
}

async fn account(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    let context = build_context(Some(user), state).await;
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

        (jar, Redirect::to("/dashboard").into_response()).into_response()
    }
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let token = jar.get("token").unwrap().value();
    AuthToken::delete_by_id(token)
        .exec(&*state.db)
        .await
        .unwrap();

    (jar, Redirect::to("/").into_response()).into_response()
}

async fn register(
    State(state): State<AppState>,
    mut jar: CookieJar,
    form: Form<LoginForm>,
) -> impl IntoResponse {
    state
        .create_user(None, &form.username, &form.password)
        .await;

    let mut cookie = Cookie::new("token", state.register_token(&form.username).await);
    cookie.set_path("/");
    jar = jar.add(cookie);

    (jar, Redirect::to("/dashboard").into_response()).into_response()
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
                Err((jar, Redirect::to("/signin").into_response()).into_response())
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
                Err((jar, Redirect::to("/signin").into_response()).into_response())
            }
        } else {
            Err(Redirect::to("/signin").into_response())
        }
    }
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
        cards.push(
            CardData::from_card(
                Card::find()
                    .filter(Expr::col(entity::card::Column::Design).eq(design.id))
                    .one(&*state.db)
                    .await
                    .unwrap()
                    .unwrap(),
                state.clone(),
                false,
            )
            .await,
        );
    }

    Json(cards)
}

async fn create_card(
    State(state): State<AppState>,
    Auth(user): Auth,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(team) = state.get_user_team(&user.username).await {
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
            team: Set(team.number),
            name: Set(bot_name.unwrap()),
            year: Set(2025),
            ..Default::default()
        })
        .exec(&*state.db)
        .await
        .unwrap();
    }
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

#[derive(Deserialize)]
struct JoinTeamData {
    team_number: String,
    join_code: Option<String>,
}

async fn join_team(
    State(state): State<AppState>,
    Auth(user): Auth,
    Form(form): Form<JoinTeamData>,
) -> impl IntoResponse {
    let team_num: i32 = form.team_number.parse().unwrap();
    let is_admin = UserTeam::find()
        .filter(Expr::col(entity::user_team::Column::Team).eq(team_num))
        .count(&*state.db)
        .await
        .unwrap()
        == 0;
    let _team = state.get_team(team_num).await;
    let active_model = entity::user_team::ActiveModel {
        user: Set(user.username),
        team: Set(team_num),
        is_admin: Set(is_admin),
    };
    UserTeam::insert(active_model)
        .exec(&*state.db)
        .await
        .unwrap();

    Redirect::to("/account")
}

async fn leave_team(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    UserTeam::delete_by_id(user.username)
        .exec(&*state.db)
        .await
        .unwrap();

    Redirect::to("/account")
}

#[derive(Deserialize)]
struct ChangePassword {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

async fn change_password(
    State(state): State<AppState>,
    Auth(user): Auth,
    Form(form): Form<ChangePassword>,
) -> impl IntoResponse {
    if state.check_user_password(&user.password, &form.current_password) {
        let salt = SaltString::generate(OsRng::default());
        let hash = argon2::Argon2::default()
            .hash_password(form.new_password.as_bytes(), &salt)
            .unwrap();
        let mut userr = user.into_active_model();
        userr.password = Set(hash.to_string());
        User::update(userr).exec(&*state.db).await.unwrap();

        Redirect::to("/account").into_response()
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Incorrect password"))
            .unwrap()
            .into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct TeamData {
    number: i32,
    name: String,
}

async fn get_team(State(state): State<AppState>, Path(team_num): Path<u32>) -> impl IntoResponse {
    let team = Team::find_by_id(team_num as i32)
        .one(&*state.db)
        .await
        .unwrap();

    if let Some(team) = team {
        Json(TeamData {
            number: team.number,
            name: team.name,
        })
        .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn modify_team(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(team_num): Path<u32>,
    Form(form): Form<TeamData>,
) -> impl IntoResponse {
    if user.is_admin
        || (state.get_user_team(&user.username).await.unwrap().number == team_num as i32
            && state.is_team_admin(&user.username).await)
    {
        let mut team = Team::find_by_id(team_num as i32)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        team.name = Set(form.name);
        Team::update(team).exec(&*state.db).await.unwrap();
    }
}

#[derive(Serialize, Deserialize)]
struct UserTeamData {
    username: String,
    is_admin: bool,
}

async fn get_team_members(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(team_num): Path<u32>,
) -> impl IntoResponse {
    if user.is_admin
        || (state.get_user_team(&user.username).await.unwrap().number == team_num as i32
            && state.is_team_admin(&user.username).await)
    {
        let links = UserTeam::find()
            .filter(Expr::col(entity::user_team::Column::Team).eq(team_num))
            .all(&*state.db)
            .await
            .unwrap();

        Json(
            links
                .into_iter()
                .map(|l| UserTeamData {
                    username: l.user,
                    is_admin: l.is_admin,
                })
                .collect::<Vec<_>>(),
        )
        .into_response()
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

async fn modify_team_member(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path((team_num, username)): Path<(u32, String)>,
    Json(data): Json<UserTeamData>,
) -> impl IntoResponse {
    if user.is_admin
        || (state.get_user_team(&user.username).await.unwrap().number == team_num as i32
            && state.is_team_admin(&user.username).await)
    {
        let mut user_team_link = UserTeam::find_by_id(username)
            .filter(Expr::col(entity::user_team::Column::Team).eq(team_num))
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();

        user_team_link.is_admin = Set(data.is_admin);

        UserTeam::update(user_team_link)
            .exec(&*state.db)
            .await
            .unwrap();

        StatusCode::OK.into_response()
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct CardData {
    card_id: Option<String>,
    card_design_id: i32,
    team_number: i32,
    year: u16,
    team_name: String,
    card_name: Option<String>,
    card_note: Option<String>,
    abilities: Option<Vec<CardAbilityData>>,
}
impl CardData {
    async fn from_card(cardd: entity::card::Model, state: AppState, unlocked: bool) -> Self {
        let design = CardDesign::find_by_id(cardd.design)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        Self {
            card_id: if unlocked { Some(cardd.id) } else { None },
            card_design_id: cardd.design,
            team_number: design.team,
            year: design.year as u16,
            team_name: state.get_team(design.team).await.name,
            card_name: if unlocked { Some(design.name) } else { None },
            card_note: if unlocked { Some(design.note) } else { None },
            abilities: if unlocked {
                Some(
                    state
                        .get_card_design_abilities(cardd.design)
                        .await
                        .into_iter()
                        .map(|a| CardAbilityData {
                            stat: a.level as u8,
                            title: a.title,
                            description: a.description,
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            },
        }
    }
}

async fn do_scan(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(cardd) = Card::find_by_id(&id).one(&*state.db).await.unwrap() {
        Scan::insert(entity::scan::ActiveModel {
            username: Set(user.username),
            card: Set(id),
            scan_time: Set(chrono::Utc::now()),
        })
        .exec(&*state.db)
        .await
        .unwrap();

        Json(CardData::from_card(cardd, state, true).await).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn get_scans(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    let scans = Scan::find()
        .filter(Expr::col(entity::scan::Column::Username).eq(user.username))
        .order_by_desc(entity::scan::Column::ScanTime)
        .all(&*state.db)
        .await
        .unwrap();

    let mut cards = Vec::new();
    for scan in scans {
        let cardd = Card::find_by_id(&scan.card)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        cards.push(CardData::from_card(cardd, state.clone(), true).await);
    }
    Json(cards).into_response()
}
