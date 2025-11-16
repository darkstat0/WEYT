use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use actix_web_lab::middleware::NormalizePath;
use std::env;
use tracing::{info, error};
use tracing_subscriber;

mod controllers;
mod services;
mod models;
mod repositories;
mod utils;
mod middleware;

use controllers::*;
use middleware::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenvy::dotenv().ok();
    
    info!("Starting Video Platform Backend");
    
    // Get configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let elasticsearch_url = env::var("ELASTICSEARCH_URL").expect("ELASTICSEARCH_URL must be set");
    
    info!("Configuration loaded:");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  Database URL: {}", database_url);
    info!("  Redis URL: {}", redis_url);
    info!("  Elasticsearch URL: {}", elasticsearch_url);
    
    // Initialize database connections
    let postgres_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to create PostgreSQL pool");
    
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
    
    let elasticsearch_client = elasticsearch::client::SyncClientBuilder::new(
        elasticsearch::http::transport::TransportBuilder::new(elasticsearch_url.parse().unwrap())
            .build()
            .unwrap()
    )
    .build()
    .unwrap();
    
    info!("Database connections established");
    
    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                postgres_pool: postgres_pool.clone(),
                redis_client: redis_client.clone(),
                elasticsearch_client: elasticsearch_client.clone(),
            }))
            .wrap(Cors::default()
                .allowed_origin_fn(|origin, _req_head| {
                    origin.as_bytes().ends_with(b".localhost")
                        || origin.as_bytes().ends_with(b".video-platform.com")
                        || origin.as_bytes().ends_with(b".uz")
                })
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Authorization", "Content-Type"])
                .max_age(3600))
            .wrap(NormalizePath::trim())
            .wrap(RateLimitMiddleware::new())
            .wrap(AuthMiddleware::new())
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/v1")
                            .configure(user_routes)
                            .configure(video_routes)
                            .configure(ai_routes)
                            .configure(auth_routes)
                            .configure(moderation_routes)
                            .configure(monetization_routes)
                    )
            )
            .service(health_check)
            .service(metrics)
    })
    .bind(format!("{}:{}", host, port))?
    .workers(num_cpus::get())
    .run();
    
    info!("Server started on http://{}:{}", host, port);
    
    server.await
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Metrics endpoint
async fn metrics() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "active_connections": 0,
        "requests_per_second": 0,
        "database_connections": 0,
        "memory_usage": 0
    }))
}

// Application state
pub struct AppState {
    pub postgres_pool: sqlx::postgres::PgPool,
    pub redis_client: redis::Client,
    pub elasticsearch_client: elasticsearch::client::SyncClient,
}

// Route configuration functions
pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_users))
            .route(web::post().to(create_user))
    )
    .service(
        web::resource("/users/{user_id}")
            .route(web::get().to(get_user))
            .route(web::put().to(update_user))
            .route(web::delete().to(delete_user))
    )
    .service(
        web::resource("/users/{user_id}/profile")
            .route(web::get().to(get_user_profile))
            .route(web::put().to(update_user_profile))
    )
    .service(
        web::resource("/users/{user_id}/videos")
            .route(web::get().to(get_user_videos))
    );
}

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/videos")
            .route(web::get().to(get_videos))
            .route(web::post().to(upload_video))
    )
    .service(
        web::resource("/videos/{video_id}")
            .route(web::get().to(get_video))
            .route(web::put().to(update_video))
            .route(web::delete().to(delete_video))
    )
    .service(
        web::resource("/videos/{video_id}/stream")
            .route(web::get().to(stream_video))
    )
    .service(
        web::resource("/videos/{video_id}/metadata")
            .route(web::get().to(get_video_metadata))
    )
    .service(
        web::resource("/videos/upload")
            .route(web::post().to(initiate_upload))
    );
}

pub fn ai_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ai/recommendations")
            .route(web::get().to(get_recommendations))
    )
    .service(
        web::resource("/ai/analyze")
            .route(web::post().to(analyze_content))
    )
    .service(
        web::resource("/ai/generate")
            .route(web::post().to(generate_content))
    )
    .service(
        web::resource("/ai/moderate")
            .route(web::post().to(moderate_content))
    );
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/auth/login")
            .route(web::post().to(login))
    )
    .service(
        web::resource("/auth/register")
            .route(web::post().to(register))
    )
    .service(
        web::resource("/auth/refresh")
            .route(web::post().to(refresh_token))
    )
    .service(
        web::resource("/auth/logout")
            .route(web::post().to(logout))
    )
    .service(
        web::resource("/auth/me")
            .route(web::get().to(get_current_user))
    );
}

pub fn moderation_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/moderation/reports")
            .route(web::get().to(get_moderation_reports))
            .route(web::post().to(create_moderation_report))
    )
    .service(
        web::resource("/moderation/reports/{report_id}")
            .route(web::put().to(update_moderation_report))
    )
    .service(
        web::resource("/moderation/content")
            .route(web::get().to(get_moderation_queue))
    );
}

pub fn monetization_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/monetization/revenue")
            .route(web::get().to(get_revenue_stats))
    )
    .service(
        web::resource("/monetization/payments")
            .route(web::post().to(process_payment))
    )
    .service(
        web::resource("/monetization/creator-payouts")
            .route(web::get().to(get_creator_payouts))
    );
}