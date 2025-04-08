use crate::{state::AppState, templates::TEMPLATES};
use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
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
        .fallback_service(ServeDir::new(PathBuf::from("public")))
}

async fn hero() -> impl IntoResponse {
    let content = TEMPLATES.render("hero.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn about() -> impl IntoResponse {
    let content = TEMPLATES.render("about.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn privacy() -> impl IntoResponse {
    let content = TEMPLATES.render("privacy.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn signin() -> impl IntoResponse {
    let content = TEMPLATES.render("signin.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn signup() -> impl IntoResponse {
    let content = TEMPLATES.render("signup.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

async fn cards() -> impl IntoResponse {
    let content = TEMPLATES.render("cards.tera", &Context::new()).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}
