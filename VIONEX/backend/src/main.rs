use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use tracing::{info, error};
use std::env;

mod config;
mod database;
mod auth;
mod users;
mod videos;
mod ai;
mod creators;
mod moderation;
mod monetization;
mod admin;
mod streaming;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Video Platform Backend");

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");
    
    // Initialize database connections
    let db_pool = database::init_db(&config).await
        .expect("Failed to initialize database connections");
    
    // Initialize Redis connection
    let redis_client = database::init_redis(&config).await
        .expect("Failed to initialize Redis");

    info!("Database and Redis connections established");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Cors::permissive())
            .service(
                web::scope("/api")
                    .service(users::register)
                    .service(users::login)
                    .service(users::get_profile)
                    .service(users::update_profile)
                    .service(videos::upload_video)
                    .service(videos::get_video)
                    .service(videos::get_video_metadata)
                    .service(videos::stream_video)
                    .service(ai::recommendations)
                    .service(ai::analyze_video)
                    .service(creators::studio_stats)
                    .service(creators::export_video)
                    .service(moderation::moderate_content)
                    .service(monetization::get_revenue)
                    .service(admin::get_dashboard)
                    .service(streaming::get_stream_manifest)
            )
            .default_service(web::to(|| HttpResponse::NotFound()))
    })
    .bind(("0.0.0.0", config.server.port))?
    .run()
    .await
}