#[macro_use]
extern crate serde;
#[macro_use]
extern crate nanoid;
#[macro_use]
extern crate log;
extern crate axum;
extern crate entity;
extern crate env_logger;
extern crate once_cell;
extern crate sea_orm;
extern crate tera;
extern crate tokio;
extern crate tower_http;

use std::env;

use state::AppState;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;

mod routes;
mod state;
mod templates;
//mod db;
mod util;

#[tokio::main]
async fn main() {
    //frcc_card_gen::test();

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);
    info!("Starting server on {}", addr);

    let comression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let state = AppState::new().await;
    let app = routes::get_router(state).layer(comression_layer);
    let listener = TcpListener::bind(&addr).await.unwrap();

    info!("Listening at {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let result = tokio::signal::ctrl_c().await;
            if let Err(e) = result {
                error!("Failed to install CTRL+C signal handler: {}", e);
                return;
            }
            debug!("CTRL+C signal received");
        })
        .await
        .unwrap();

    info!("Shutting down");
}
