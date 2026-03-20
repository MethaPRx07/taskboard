mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repository;
mod routes;
mod state;
mod utils;

use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web::ResponseError;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load config from environment
    let config = config::Config::from_env();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Connecting to database...");
    let pool = db::create_pool(&config.database_url).await;

    // Run pending migrations automatically
    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let host = config.host.clone();
    let port = config.port;
    let state = web::Data::new(AppState { db: pool, config });

    log::info!("🚀 Server starting on http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(
                // Return JSON error for malformed JSON bodies
                web::JsonConfig::default()
                    .error_handler(|err, _req| {
                        let response = errors::AppError::BadRequest(err.to_string())
                            .error_response();
                        actix_web::error::InternalError::from_response(err, response).into()
                    }),
            )
            .wrap(Logger::new(
                "%r → %s (%D ms) [%a]",
            ))
            .configure(routes::configure)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
