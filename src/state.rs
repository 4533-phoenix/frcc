use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use entity::{auth_token, prelude::*, user};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveValue::Set, Database, DatabaseConnection, EntityTrait,
};

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Arc::new(
            Database::connect("sqlite:./test.db?mode=rwc")
                .await
                .unwrap(),
        );

        Migrator::up(&*db, None).await.unwrap();

        Self { db }
    }

    pub async fn get_team(&self, team_number: i32) -> entity::team::Model {
        if let Some(team) = Team::find_by_id(team_number).one(&*self.db).await.unwrap() {
            team
        } else {
            let active_model = entity::team::ActiveModel {
                number: Set(team_number as i32),
                name: Set(String::new()),
            };

            Team::insert(active_model).exec(&*self.db).await.unwrap();

            Team::find_by_id(team_number).one(&*self.db).await.unwrap().unwrap()
        }
    }

    pub async fn get_user_team(&self, username: &str) -> Option<entity::team::Model> {
        if let Some(user_team) = UserTeam::find_by_id(username).one(&*self.db).await.unwrap() {
            Some(self.get_team(user_team.team).await)
        } else {
            None
        }
    }
    pub async fn is_team_admin(&self, username: &str) -> bool {
        if let Some(user_team) = UserTeam::find_by_id(username).one(&*self.db).await.unwrap() {
            user_team.is_admin
        } else {
            false
        }
    }

    pub fn check_user_password(&self, hash: &str, password: &str) -> bool {
        dbg!(password);
        if Argon2::default()
            .verify_password(password.as_bytes(), &PasswordHash::new(hash).unwrap())
            .is_ok()
        {
            true
        } else {
            false
        }
    }

    pub async fn create_user(&self, invite_code: Option<String>, username: &str, password: &str) {
        let salt = SaltString::generate(OsRng::default());
        let hash = argon2::Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap();
        let user = entity::user::ActiveModel {
            username: Set(username.to_string()),
            password: Set(hash.to_string()),
            ..Default::default()
        };
        User::insert(user).exec(&*self.db).await.unwrap();
    }

    pub async fn create_invite_code(&self, inviter: &str) -> Option<String> {
        let code = nanoid!(64);

        let invite = entity::invite::ActiveModel {
            code: Set(code.clone()),
            inviter: Set(inviter.to_string()),
        };
        Invite::insert(invite).exec(&*self.db).await.unwrap();

        Some(code)
    }

    pub async fn is_valid_invite_code(&self, code: &str) -> bool {
        Invite::find_by_id(code).one(&*self.db).await.ok().is_some()
    }

    pub async fn register_token(&self, username: &str) -> String {
        let token = nanoid!(64);
        AuthToken::insert(auth_token::ActiveModel {
            user: Set(username.to_string()),
            token: Set(token.clone()),
        })
        .exec(&*self.db)
        .await
        .expect("Failed to create auth token");
        token
    }

    pub async fn get_user_for_token(&self, token: &str) -> Option<user::Model> {
        match AuthToken::find_by_id(token).one(&*self.db).await.unwrap() {
            Some(token) => User::find_by_id(token.user).one(&*self.db).await.unwrap(),
            None => None,
        }
    }

    pub async fn get_card_design_abilities(&self, card_design_id: i32) -> Vec<entity::card_ability::Model> {
        CardAbility::find_by_id(card_design_id)
            .all(&*self.db)
            .await
            .unwrap()
    }
}
