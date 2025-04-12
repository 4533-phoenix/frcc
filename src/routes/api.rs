use crate::{
    routes::structs::{CardAbilityData, UserData},
    state::AppState,
    util::optimize_and_save_model,
};
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
};
use axum_extra::extract::{CookieJar, Multipart, cookie::Cookie};
use chrono::Datelike;
use entity::{card_design, prelude::*, user};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder,
    prelude::Expr,
};

use super::{
    structs::{
        CardData, ChangePassword, GetCardsParams, JoinTeamData, LoginForm, TeamData, UserTeamData,
    },
    util::Auth,
};

pub async fn login(
    State(state): State<AppState>,
    mut jar: CookieJar,
    form: Form<LoginForm>,
) -> impl IntoResponse {
    if let Some(user) = User::find_by_id(&form.username)
        .one(&*state.db)
        .await
        .unwrap()
    {
        if !state.check_user_password(&user.password, &form.password) {
            Redirect::to("/signin?error=invalid").into_response()
        } else {
            let mut cookie = Cookie::new("token", state.register_token(&user.username).await);
            cookie.set_path("/");
            jar = jar.add(cookie);

            (jar, Redirect::to("/dashboard").into_response()).into_response()
        }
    } else {
        Redirect::to("/signin?error=invalid").into_response()
    }
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let token = jar.get("token").unwrap().value();
    AuthToken::delete_by_id(token)
        .exec(&*state.db)
        .await
        .unwrap();

    (jar, Redirect::to("/").into_response()).into_response()
}

pub async fn register(
    State(state): State<AppState>,
    mut jar: CookieJar,
    form: Form<LoginForm>,
) -> impl IntoResponse {
    if User::find_by_id(&form.username)
        .one(&*state.db)
        .await
        .unwrap()
        .is_some()
    {
        return Redirect::to("/signup?error=taken").into_response();
    }

    state
        .create_user(None, &form.username, &form.password)
        .await;

    let mut cookie = Cookie::new("token", state.register_token(&form.username).await);
    cookie.set_path("/");
    jar = jar.add(cookie);

    (jar, Redirect::to("/dashboard").into_response()).into_response()
}

pub async fn create_invite_code(
    Auth(user): Auth,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if user.is_admin || user.is_verified {
        let invite_code = state.create_invite_code(&user.username).await.unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(invite_code.to_string())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not allowed to create invite codes".to_string())
            .unwrap()
    }
}

pub async fn get_cards(
    State(state): State<AppState>,
    Query(params): Query<GetCardsParams>,
) -> impl IntoResponse {
    let mut cards = Vec::new();

    if let Some(user) = params.user {
        let scans = Scan::find()
            .filter(Expr::col(entity::scan::Column::Username).eq(user))
            .all(&*state.db)
            .await
            .unwrap();

        for scan in scans {
            cards.push(
                CardData::from_card(
                    Card::find_by_id(scan.card)
                        .one(&*state.db)
                        .await
                        .unwrap()
                        .unwrap(),
                    state.clone(),
                    true,
                )
                .await,
            );
        }
    } else {
        let mut sel = CardDesign::find();

        if let Some(team) = params.team {
            sel = sel.filter(Expr::col(entity::card_design::Column::Team).eq(team));
        }

        if let Some(year) = params.year {
            sel = sel.filter(Expr::col(entity::card_design::Column::Year).eq(year));
        }

        let designs = sel.all(&*state.db).await.unwrap();

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
    }

    Json(cards)
}

pub async fn create_card(
    State(state): State<AppState>,
    Auth(user): Auth,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(team) = state.get_user_team(&user.username).await {
        let id = nanoid!(33);

        let mut bot_name = None;
        let mut note = None;
        let mut abilities = None;
        let mut photo = None;
        let mut model = None;

        while let Some(field) = multipart.next_field().await.unwrap() {
            match field.name() {
                Some("bot_name") => {
                    bot_name = field.text().await.ok();
                }
                Some("note") => {
                    note = field.text().await.ok();
                }
                Some("abilities") => {
                    if field.content_type() == Some("application/json") {
                        abilities = field.text().await.ok();
                    }
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

        let abilities: Vec<CardAbilityData> = if let Some(abilities_str) = abilities {
            serde_json::from_str(&abilities_str).unwrap()
        } else {
            Vec::new()
        };

        optimize_and_save_model(id.clone(), model.unwrap().to_vec());

        let design = CardDesign::insert(card_design::ActiveModel {
            team: Set(team.number),
            year: Set(chrono::Utc::now().year() as i16),
            name: Set(bot_name.unwrap()),
            note: Set(note.unwrap_or_default()),
            ..Default::default()
        })
        .exec(&*state.db)
        .await
        .unwrap();

        for ability in abilities {
            CardAbility::insert(entity::card_ability::ActiveModel {
                card: Set(design.last_insert_id),
                level: Set(ability.level as i8),
                amount: Set(ability.amount),
                title: Set(ability.title),
                description: Set(ability.description),
            })
            .exec(&*state.db)
            .await
            .unwrap();
        }
    }
}

pub async fn get_user(
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
            team: state
                .get_user_team(&user.username)
                .await
                .map(|t| t.number.to_string()),
            is_team_admin: Some(state.is_team_admin(&user.username).await),
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

pub async fn get_users(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    if user.is_admin {
        let mut users = Vec::new();
        for user in User::find().all(&*state.db).await.unwrap() {
            users.push(UserData {
                username: user.username.clone(),
                is_admin: user.is_admin,
                is_verified: user.is_verified,
                team: state
                    .get_user_team(&user.username)
                    .await
                    .map(|t| t.number.to_string()),
                is_team_admin: Some(state.is_team_admin(&user.username).await),
            });
        }

        Json(users).into_response()
    } else {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("You are not an admin".to_string())
            .unwrap()
            .into_response()
    }
}

pub async fn modify_user(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(username): Path<String>,
    Json(data): Json<UserData>,
) -> impl IntoResponse {
    if user.is_admin {
        let mut user = User::find_by_id(&username)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        user.is_admin = Set(data.is_admin);
        user.is_verified = Set(data.is_verified);
        User::update(user).exec(&*state.db).await.unwrap();

        state
            .set_user_team(
                &username,
                data.team.map(|t| t.parse().unwrap()),
                data.is_team_admin.unwrap(),
                None,
            )
            .await;

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

pub async fn join_team(
    State(state): State<AppState>,
    Auth(user): Auth,
    Form(form): Form<JoinTeamData>,
) -> impl IntoResponse {
    let team_num: i32 = form.team_number.parse().unwrap();

    // User will become an admin if they:
    //  1. used an invite code
    //  2. it has not already been used
    let is_admin = if let Some(code) = form.join_code.clone() {
        if code.is_empty() {
            false
        } else {
            UserTeam::find()
                .filter(Expr::col(entity::user_team::Column::Invite).eq(code))
                .count(&*state.db)
                .await
                .unwrap()
                == 0
        }
    } else {
        false
    };

    // Ensure the team exists
    let _team = state.get_team(team_num).await;

    let active_model = entity::user_team::ActiveModel {
        user: Set(user.username),
        team: Set(team_num),
        is_admin: Set(is_admin),
        invite: if is_admin {
            Set(form.join_code)
        } else {
            Set(None)
        },
    };

    UserTeam::insert(active_model)
        .exec(&*state.db)
        .await
        .unwrap();

    Redirect::to("/account")
}

pub async fn leave_team(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
    // If the user used an invite to become a team admin, destroy the invite so it can't be reused
    if let Some(invite) = UserTeam::find_by_id(&user.username)
        .one(&*state.db)
        .await
        .unwrap()
        .unwrap()
        .invite
    {
        Invite::delete_by_id(invite).exec(&*state.db).await.unwrap();
    }

    UserTeam::delete_by_id(user.username)
        .exec(&*state.db)
        .await
        .unwrap();

    Redirect::to("/account")
}

pub async fn change_password(
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

pub async fn get_team(
    State(state): State<AppState>,
    Path(team_num): Path<u32>,
) -> impl IntoResponse {
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

pub async fn modify_team(
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

pub async fn get_team_members(
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

pub async fn modify_team_member(
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

pub async fn do_scan(
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

pub async fn get_scans(State(state): State<AppState>, Auth(user): Auth) -> impl IntoResponse {
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
