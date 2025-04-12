use crate::{state::AppState, templates::TEMPLATES};
use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use entity::prelude::*;
use sea_orm::{prelude::Expr, EntityTrait, QueryFilter};
use tera::Context;

use super::util::{build_context, Auth, IsAuth};

// Helper function to ensure consistent context variables across all pages
async fn create_standard_context(
    is_auth: bool,
    user: Option<entity::user::Model>,
    state: Option<AppState>,
) -> Context {
    let mut context = Context::new();
    context.insert("is_auth", &is_auth);

    // Add site name and version
    context.insert("site_name", "FRCC");
    context.insert("site_version", "1.0.0");

    if let Some(user) = user {
        if let Some(state) = state {
            return build_context(Some(user), state).await;
        } else {
            context.insert("username", &user.username);
            context.insert("is_site_admin", &user.is_admin);
        }
    }

    context
}

pub async fn scan(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("scan.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn hero(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    if is_auth {
        Redirect::to("/dashboard").into_response()
    } else {
        let context = create_standard_context(is_auth, None, None).await;
        let content = TEMPLATES.render("hero.tera", &context).unwrap();
        Response::builder()
            .header("Content-Type", "text/html")
            .body(content)
            .unwrap()
            .into_response()
    }
}

pub async fn about(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("about.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn privacy(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("privacy.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn signin(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("signin.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn signup(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("signup.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn cards(IsAuth(is_auth): IsAuth) -> impl IntoResponse {
    let context = create_standard_context(is_auth, None, None).await;
    let content = TEMPLATES.render("cards.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn dashboard(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    // In a real application, we would fetch this data from a database
    // based on the authenticated user's session
    let mut context = create_standard_context(true, Some(user.clone()), Some(state.clone())).await;

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

pub async fn admin(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    // Check if the user is a site admin
    if !user.is_admin {
        return Redirect::to("/dashboard").into_response();
    }
    let context = create_standard_context(true, Some(user), Some(state)).await;

    let content = TEMPLATES.render("admin.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap()
}

pub async fn account(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    let context = create_standard_context(true, Some(user), Some(state)).await;
    let content = TEMPLATES.render("account.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}

pub async fn edit_cards(Auth(user): Auth, State(state): State<AppState>) -> impl IntoResponse {
    // Check if the user is a team admin
    if !state.is_team_admin(&user.username).await {
        return Redirect::to("/dashboard").into_response();
    }
    
    let context = create_standard_context(true, Some(user), Some(state)).await;
    let content = TEMPLATES.render("edit_cards.tera", &context).unwrap();
    Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap()
}
