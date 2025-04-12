use crate::state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post, put},
};

mod api;
mod frontend;
mod structs;
mod util;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(frontend::hero))
        .route("/about", get(frontend::about))
        .route("/privacy", get(frontend::privacy))
        .route("/signin", get(frontend::signin))
        .route("/signup", get(frontend::signup))
        .route("/cards", get(frontend::cards))
        .route("/scan", get(frontend::scan))
        .route("/dashboard", get(frontend::dashboard))
        .route("/admin", get(frontend::admin))
        .route("/account", get(frontend::account))
        .route("/edit_cards", get(frontend::edit_cards))
        .with_state(state.clone())
        .nest("/api", get_api_router(state))
        .fallback_service(get(util::static_handler))
}

pub fn get_api_router(state: AppState) -> Router {
    Router::new()
        .route("/login", post(api::login))
        .route("/logout", get(api::logout))
        .route("/register", post(api::register))
        .route("/cards", get(api::get_cards).post(api::create_card))
        .route("/design/{id}", get(api::get_design)) //.put(api::modify_card))
        .route("/scans", put(api::do_scan).get(api::get_scans))
        .route("/user/{username}", get(api::get_user).put(api::modify_user))
        .route("/users", get(api::get_users))
        .route("/invites", put(api::create_invite_code))
        .route("/account/join_team", post(api::join_team))
        .route("/account/leave_team", post(api::leave_team))
        .route("/account/change_password", post(api::change_password))
        .route(
            "/team/{team_num}",
            get(api::get_team).post(api::modify_team),
        )
        .route("/team/{team_num}/members", get(api::get_team_members))
        .route(
            "/team/{team_num}/member/{username}",
            put(api::modify_team_member),
        )
        .with_state(state)
        .layer(DefaultBodyLimit::disable())
}
