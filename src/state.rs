use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use toasty::{stmt::Select, Model};

use crate::db::{init_db, AuthToken, Invite, User};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<toasty::Db>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Arc::new(init_db().await);
        let state = Self { db };

        if User::get_by_username(&state.db, "admin").await.is_err() {
            state.create_user(None, "admin", "admin").await;
        }

        state
    }

    pub fn check_user_password(&self, hash: &str, password: &str) -> bool {
        dbg!(password);
            if Argon2::default().verify_password(password.as_bytes(), &PasswordHash::new(hash).unwrap()).is_ok() {
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
        User::create().username(username).invited_with_code(invite_code).password(&hash.to_string()).exec(&self.db).await.unwrap();
    }

    pub async fn create_invite_code(&self, inviter: &str) -> Option<String> {
        let code = nanoid!(64);

        Invite::create().inviter_username(inviter).invite_code(&code).exec(&self.db).await.unwrap();

        Some(code)
    }

    pub async fn is_valid_invite_code(&self, code: &str) -> bool {
        Invite::get_by_invite_code(&self.db, code).await.ok().is_some()
    }

    pub async fn register_token(&self, user: &User) -> String {
        let token = nanoid!(64);
        user.auth_tokens().create().token(&token).exec(&self.db).await.expect("Failed to create auth token");
        token
    }

    pub async fn get_user_for_token(&self, token: &str) -> Option<User> {
        match AuthToken::get_by_token(&self.db, token).await.ok() {
            Some(token) => token.user().get(&self.db).await.ok(),
            None => None,
        }
    }
}
