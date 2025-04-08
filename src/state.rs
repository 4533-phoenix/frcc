use sled::{Config, Db, Mode};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn default() -> Self {
        let db = Config::default()
            .path("db")
            .flush_every_ms(Some(1000))
            .mode(Mode::HighThroughput)
            .open()
            .unwrap();

        Self::new(db)
    }
}
