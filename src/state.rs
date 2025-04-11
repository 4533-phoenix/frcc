use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use entity::{auth_token, prelude::*, user};
use sea_orm::{
    ActiveValue::Set, Database, DatabaseConnection, EntityTrait, QueryFilter, prelude::Expr,
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
        let state = Self { db };

        if User::find_by_id("admin")
            .one(&*state.db)
            .await
            .unwrap()
            .is_none()
        {
            state.create_user(None, "admin", "admin").await;
        }

        state
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
}
