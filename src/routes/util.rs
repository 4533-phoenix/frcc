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

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
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

pub async fn build_context(user: Option<entity::user::Model>, state: AppState) -> Context {
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

