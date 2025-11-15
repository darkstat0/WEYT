use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::{auth, database, utils};
use crate::database::DbPool;
use crate::config::Config;
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminDashboardRequest {
    pub time_range: String, // "7d", "30d", "90d", "1y"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminDashboard {
    pub overview: PlatformOverview,
    pub user_stats: UserStats,
    pub content_stats: ContentStats,
    pub revenue_stats: RevenueStats,
    pub system_health: SystemHealth,
    pub alerts: Vec<SystemAlert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformOverview {
    pub total_users: i64,
    pub active_users: i64,
    pub total_videos: i64,
    pub total_views: i64,
    pub total_revenue: f64,
    pub growth_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub new_users: i64,
    pub premium_users: i64,
    pub creator_users: i64,
    pub top_countries: Vec<CountryData>,
    pub user_activity: Vec<DailyActivity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentStats {
    pub new_videos: i64,
    pub processed_videos: i64,
    pub failed_videos: i64,
    pub total_storage: i64,
    pub avg_processing_time: f64,
    pub popular_categories: Vec<CategoryData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueStats {
    pub total_revenue: f64,
    pub by_source: HashMap<String, f64>,
    pub by_region: HashMap<String, f64>,
    pub payout_pending: f64,
    pub avg_rpm: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
    pub active_streams: i64,
    pub database_connections: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAlert {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryData {
    pub country: String,
    pub user_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub active_users: i64,
    pub new_users: i64,
    pub new_videos: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryData {
    pub category: String,
    pub video_count: i64,
    pub views: i64,
    pub engagement_rate: f64,
}

// Get admin dashboard
pub async fn get_dashboard(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<AdminDashboardRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let dashboard = get_admin_dashboard_data(pool, &req.time_range).await;

    match dashboard {
        Ok(dashboard_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Admin dashboard retrieved".to_string(),
                data: Some(serde_json::to_value(dashboard_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get admin dashboard: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get dashboard data".to_string(),
                errors: None,
            }))
        }
    }
}

// Get admin dashboard data
async fn get_admin_dashboard_data(
    pool: &DbPool,
    time_range: &str,
) -> Result<AdminDashboard, Box<dyn std::error::Error + Send + Sync>> {
    let (start_date, end_date) = calculate_date_range(time_range);

    let overview = get_platform_overview(pool, start_date, end_date).await?;
    let user_stats = get_user_statistics(pool, start_date, end_date).await?;
    let content_stats = get_content_statistics(pool, start_date, end_date).await?;
    let revenue_stats = get_revenue_statistics(pool, start_date, end_date).await?;
    let system_health = get_system_health(pool).await?;
    let alerts = get_system_alerts(pool).await?;

    Ok(AdminDashboard {
        overview,
        user_stats,
        content_stats,
        revenue_stats,
        system_health,
        alerts,
    })
}

// Calculate date range
fn calculate_date_range(time_range: &str) -> (DateTime<Utc>, DateTime<Utc>) {
    let end_date = Utc::now();
    let start_date = match time_range {
        "7d" => end_date - chrono::Duration::days(7),
        "30d" => end_date - chrono::Duration::days(30),
        "90d" => end_date - chrono::Duration::days(90),
        "1y" => end_date - chrono::Duration::days(365),
        _ => end_date - chrono::Duration::days(30), // Default to 30 days
    };
    (start_date, end_date)
}

// Get platform overview
async fn get_platform_overview(
    pool: &DbPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<PlatformOverview, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        SELECT 
            COUNT(*) as total_users,
            COUNT(CASE WHEN last_login > $1 THEN 1 END) as active_users,
            COUNT(DISTINCT v.id) as total_videos,
            COALESCE(SUM(v.view_count), 0) as total_views,
            COALESCE(SUM(cr.amount), 0) as total_revenue
        FROM users u
        LEFT JOIN videos v ON u.id = v.user_id
        LEFT JOIN creator_revenue cr ON v.id = cr.video_id
        WHERE u.created_at <= $2
    "#;

    let result = sqlx::query!(query, start_date, end_date)
        .fetch_one(pool)
        .await?;

    // Calculate growth rate (simplified)
    let growth_rate = 15.0; // Placeholder

    Ok(PlatformOverview {
        total_users: result.total_users.unwrap_or(0),
        active_users: result.active_users.unwrap_or(0),
        total_videos: result.total_videos.unwrap_or(0),
        total_views: result.total_views.unwrap_or(0),
        total_revenue: result.total_revenue.unwrap_or(0.0),
        growth_rate,
    })
}

// Get user statistics
async fn get_user_statistics(
    pool: &DbPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<UserStats, Box<dyn std::error::Error + Send + Sync>> {
    // Get new users
    let new_users = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE created_at BETWEEN $1 AND $2",
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?
    .count.unwrap_or(0);

    // Get premium users
    let premium_users = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE is_premium = TRUE"
    )
    .fetch_one(pool)
    .await?
    .count.unwrap_or(0);

    // Get creator users
    let creator_users = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE role = 'creator'"
    )
    .fetch_one(pool)
    .await?
    .count.unwrap_or(0);

    // Get top countries
    let top_countries = sqlx::query_as!(
        CountryData,
        r#"
        SELECT 
            CASE 
                WHEN ip_address::text LIKE 'US%' THEN 'United States'
                WHEN ip_address::text LIKE 'GB%' THEN 'United Kingdom'
                WHEN ip_address::text LIKE 'CA%' THEN 'Canada'
                WHEN ip_address::text LIKE 'AU%' THEN 'Australia'
                WHEN ip_address::text LIKE 'DE%' THEN 'Germany'
                ELSE 'Other'
            END as country,
            COUNT(*) as user_count,
            COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percentage
        FROM users u
        LEFT JOIN video_views vv ON u.id = vv.user_id
        WHERE vv.created_at BETWEEN $1 AND $2
        GROUP BY country
        ORDER BY user_count DESC
        LIMIT 5
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    // Get daily activity
    let user_activity = sqlx::query_as!(
        DailyActivity,
        r#"
        SELECT 
            DATE(created_at) as date,
            COUNT(*) as new_users,
            COUNT(CASE WHEN last_login > DATE(created_at) + INTERVAL '1 day' THEN 1 END) as active_users,
            COUNT(DISTINCT v.id) as new_videos
        FROM users u
        LEFT JOIN videos v ON u.id = v.user_id
        WHERE created_at BETWEEN $1 AND $2
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(UserStats {
        new_users,
        premium_users,
        creator_users,
        top_countries,
        user_activity,
    })
}

// Get content statistics
async fn get_content_statistics(
    pool: &DbPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<ContentStats, Box<dyn std::error::Error + Send + Sync>> {
    // Get video statistics
    let video_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as new_videos,
            COUNT(CASE WHEN status = 'ready' THEN 1 END) as processed_videos,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_videos,
            COALESCE(SUM(size), 0) as total_storage
        FROM videos
        WHERE created_at BETWEEN $1 AND $2
        "#,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    // Get average processing time
    let avg_processing_time = sqlx::query!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (processing_completed_at - created_at))) as avg_time
        FROM videos
        WHERE status = 'ready' AND processing_completed_at IS NOT NULL
          AND created_at BETWEEN $1 AND $2
        "#,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    // Get popular categories
    let popular_categories = sqlx::query_as!(
        CategoryData,
        r#"
        SELECT 
            topic_category as category,
            COUNT(*) as video_count,
            COALESCE(SUM(view_count), 0) as views,
            AVG((like_count + comment_count + share_count) * 100.0 / NULLIF(view_count, 0)) as engagement_rate
        FROM videos
        WHERE status = 'ready' AND topic_category IS NOT NULL
          AND created_at BETWEEN $1 AND $2
        GROUP BY topic_category
        ORDER BY views DESC
        LIMIT 10
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(ContentStats {
        new_videos: video_stats.new_videos.unwrap_or(0),
        processed_videos: video_stats.processed_videos.unwrap_or(0),
        failed_videos: video_stats.failed_videos.unwrap_or(0),
        total_storage: video_stats.total_storage.unwrap_or(0),
        avg_processing_time: avg_processing_time.avg_time.unwrap_or(0.0),
        popular_categories,
    })
}

// Get revenue statistics
async fn get_revenue_statistics(
    pool: &DbPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<RevenueStats, Box<dyn std::error::Error + Send + Sync>> {
    // Get total revenue
    let total_revenue = sqlx::query!(
        "SELECT COALESCE(SUM(amount), 0) as total FROM creator_revenue WHERE created_at BETWEEN $1 AND $2",
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?
    .total.unwrap_or(0.0);

    // Get revenue by source
    let by_source = sqlx::query!(
        r#"
        SELECT revenue_type, COALESCE(SUM(amount), 0) as total
        FROM creator_revenue
        WHERE created_at BETWEEN $1 AND $2
        GROUP BY revenue_type
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let mut source_map = HashMap::new();
    for result in by_source {
        source_map.insert(result.revenue_type, result.total.unwrap_or(0.0));
    }

    // Get pending payouts
    let payout_pending = sqlx::query!(
        "SELECT COALESCE(SUM(amount), 0) as total FROM creator_revenue WHERE status = 'processing'"
    )
    .fetch_one(pool)
    .await?
    .total.unwrap_or(0.0);

    // Get average RPM
    let avg_rpm = sqlx::query!(
        r#"
        SELECT AVG(amount * 1000.0 / NULLIF(view_count, 0)) as avg_rpm
        FROM creator_revenue cr
        JOIN videos v ON cr.video_id = v.id
        WHERE cr.created_at BETWEEN $1 AND $2 AND v.view_count > 0
        "#,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?
    .avg_rpm
    .unwrap_or(0.0);

    Ok(RevenueStats {
        total_revenue,
        by_source: source_map,
        by_region: HashMap::new(), // Placeholder
        payout_pending,
        avg_rpm,
    })
}

// Get system health
async fn get_system_health(
    pool: &DbPool,
) -> Result<SystemHealth, Box<dyn std::error::Error + Send + Sync>> {
    // Get database connections
    let db_connections = sqlx::query!(
        "SELECT COUNT(*) as count FROM pg_stat_activity WHERE state = 'active'"
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0) as i32;

    // Placeholder values for system metrics
    Ok(SystemHealth {
        cpu_usage: 45.0,
        memory_usage: 60.0,
        disk_usage: 75.0,
        network_usage: 30.0,
        active_streams: 150,
        database_connections: db_connections,
    })
}

// Get system alerts
async fn get_system_alerts(
    pool: &DbPool,
) -> Result<Vec<SystemAlert>, Box<dyn std::error::Error + Send + Sync>> {
    // In production, this would query system alerts table
    // For now, return placeholder alerts
    Ok(vec
![
        SystemAlert {
            id: Uuid::new_v4(),
            alert_type: "storage".to_string(),
            severity: "warning".to_string(),
            message: "Storage space running low".to_string(),
            timestamp: Utc::now(),
            resolved: false,
        },
        SystemAlert {
            id: Uuid::new_v4(),
            alert_type: "performance".to_string(),
            severity: "info".to_string(),
            message: "Processing queue building up".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(30),
            resolved: false,
        },
    ])
}

// Manage users
pub async fn manage_users(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<ManageUserRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    match req.action.as_str() {
        "ban" => {
            if let Err(e) = ban_user(pool, req.user_id).await {
                eprintln!("Failed to ban user: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to ban user".to_string(),
                    errors: None,
                }));
            }
        }
        "unban" => {
            if let Err(e) = unban_user(pool, req.user_id).await {
                eprintln!("Failed to unban user: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to unban user".to_string(),
                    errors: None,
                }));
            }
        }
        "verify" => {
            if let Err(e) = verify_user(pool, req.user_id).await {
                eprintln!("Failed to verify user: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to verify user".to_string(),
                    errors: None,
                }));
            }
        }
        "premium" => {
            if let Err(e) = make_premium_user(pool, req.user_id, req.premium_months.unwrap_or(12)).await {
                eprintln!("Failed to make user premium: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to make user premium".to_string(),
                    errors: None,
                }));
            }
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Invalid action".to_string(),
                errors: None,
            }));
        }
    }

    // Log admin action
    if let Err(e) = log_admin_action(pool, user.id, req.user_id, &req.action, &req.reason).await {
        eprintln!("Failed to log admin action: {}", e);
    }

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "User action completed successfully".to_string(),
        data: None,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManageUserRequest {
    pub user_id: Uuid,
    pub action: String, // "ban", "unban", "verify", "premium"
    pub reason: Option<String>,
    pub premium_months: Option<i32>,
}

// Ban user
async fn ban_user(pool: &DbPool, user_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        "UPDATE users SET is_active = FALSE, updated_at = $1 WHERE id = $2",
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    // Delete user content
    sqlx::query!(
        "UPDATE videos SET status = 'deleted' WHERE user_id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Unban user
async fn unban_user(pool: &DbPool, user_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        "UPDATE users SET is_active = TRUE, updated_at = $1 WHERE id = $2",
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Verify user
async fn verify_user(pool: &DbPool, user_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        "UPDATE users SET is_verified = TRUE, updated_at = $1 WHERE id = $2",
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Make user premium
async fn make_premium_user(
    pool: &DbPool,
    user_id: Uuid,
    months: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let expiry_date = Utc::now() + chrono::Duration::days(months * 30);

    sqlx::query!(
        "UPDATE users SET is_premium = TRUE, updated_at = $1 WHERE id = $2",
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    // Record premium subscription
    sqlx::query!(
        r#"
        INSERT INTO premium_subscriptions (user_id, months, expiry_date, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        months,
        expiry_date,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Log admin action
async fn log_admin_action(
    pool: &DbPool,
    admin_id: Uuid,
    target_user_id: Uuid,
    action: &str,
    reason: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO admin_actions (admin_id, target_user_id, action, reason, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        admin_id,
        target_user_id,
        action,
        reason,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// System monitoring
pub async fn get_system_metrics(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let metrics = get_detailed_system_metrics(pool).await;

    match metrics {
        Ok(metrics_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "System metrics retrieved".to_string(),
                data: Some(serde_json::to_value(metrics_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get system metrics: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get system metrics".to_string(),
                errors: None,
            }))
        }
    }
}

// Get detailed system metrics
async fn get_detailed_system_metrics(
    pool: &DbPool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    // Get database metrics
    let db_metrics = sqlx::query!(
        "SELECT 
            COUNT(*) as total_connections,
            COUNT(CASE WHERE state = 'active' THEN 1 END) as active_connections,
            COUNT(CASE WHERE state = 'idle' THEN 1 END) as idle_connections
        FROM pg_stat_activity"
    )
    .fetch_one(pool)
    .await?;

    // Get video processing metrics
    let processing_metrics = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_queue,
            COUNT(CASE WHERE status = 'processing' THEN 1 END) as processing,
            COUNT(CASE WHERE status = 'failed' THEN 1 END) as failed,
            AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_wait_time
        FROM videos
        WHERE status IN ('processing', 'failed')
        "#
    )
    .fetch_one(pool)
    .await?;

    // Get storage metrics
    let storage_metrics = sqlx::query!(
        "SELECT COALESCE(SUM(size), 0) as total_storage FROM videos WHERE status = 'ready'"
    )
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "database": {
            "total_connections": db_metrics.total_connections.unwrap_or(0),
            "active_connections": db_metrics.active_connections.unwrap_or(0),
            "idle_connections": db_metrics.idle_connections.unwrap_or(0),
            "connection_efficiency": db_metrics.active_connections.unwrap_or(0) as f64 / db_metrics.total_connections.unwrap_or(1) as f64 * 100.0
        },
        "video_processing": {
            "total_queue": processing_metrics.total_queue.unwrap_or(0),
            "processing": processing_metrics.processing.unwrap_or(0),
            "failed": processing_metrics.failed.unwrap_or(0),
            "failure_rate": processing_metrics.failed.unwrap_or(0) as f64 / processing_metrics.total_queue.unwrap_or(1) as f64 * 100.0,
            "avg_wait_time": processing_metrics.avg_wait_time.unwrap_or(0.0)
        },
        "storage": {
            "total_storage_gb": storage_metrics.total_storage.unwrap_or(0) as f64 / (1024.0 * 1024.0 * 1024.0),
            "estimated_growth_rate": 2.5 // GB per day placeholder
        },
        "performance": {
            "uptime": "99.9%", // Placeholder
            "response_time_avg": 150, // ms
            "error_rate": 0.1 // %
        }
    }))
}

// Security monitoring
pub async fn get_security_logs(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    params: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as i32;
    let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as i32;
    let log_type = params.get("type").and_then(|v| v.as_str());

    let logs = get_security_logs_filtered(pool, log_type, limit, offset).await;

    match logs {
        Ok(logs_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Security logs retrieved".to_string(),
                data: Some(serde_json::to_value(logs_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get security logs: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get security logs".to_string(),
                errors: None,
            }))
        }
    }
}

// Get filtered security logs
async fn get_security_logs_filtered(
    pool: &DbPool,
    log_type: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<SecurityLog>, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = String::from(
        r#"
        SELECT 
            id, user_id, action, ip_address, user_agent, 
            details, created_at, severity
        FROM security_logs
        WHERE 1=1
        "#,
    );

    let mut query_params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![];

    if let Some(log_type) = log_type {
        query.push_str(&format!(" AND action = ${}", query_params.len() + 1));
        query_params.push(Box::new(log_type.to_string()));
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", query_params.len() + 1, query_params.len() + 2));
    query_params.push(Box::new(limit));
    query_params.push(Box::new(offset));

    let logs = sqlx::query_as::<_, SecurityLog>(query.as_str())
        .bind_all(query_params)
        .fetch_all(pool)
        .await?;

    Ok(logs)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct SecurityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub severity: String,
}

// Configuration management
pub async fn update_system_config(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<UpdateConfigRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin
    if user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    // Validate configuration
    if !is_valid_config(&req.config) {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Invalid configuration".to_string(),
            errors: None,
        }));
    }

    // Update configuration
    if let Err(e) = update_system_configuration(pool, &req.config).await {
        eprintln!("Failed to update system config: {}", e);
        return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
            success: false,
            message: "Failed to update configuration".to_string(),
            errors: None,
        }));
    }

    // Log configuration change
    if let Err(e) = log_config_change(pool, user.id, &req.config).await {
        eprintln!("Failed to log config change: {}", e);
    }

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "System configuration updated".to_string(),
        data: None,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub config: serde_json::Value,
}

// Validate configuration
fn is_valid_config(config: &serde_json::Value) -> bool {
    // Basic validation - in production, this would be more comprehensive
    config.is_object() && config.get("max_upload_size").is_some()
}

// Update system configuration
async fn update_system_configuration(
    pool: &DbPool,
    config: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        "INSERT INTO system_config (config, updated_at) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET config = $1, updated_at = $2",
        config,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Log configuration change
async fn log_config_change(
    pool: &DbPool,
    admin_id: Uuid,
    config: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO config_changes (admin_id, config_change, created_at)
        VALUES ($1, $2, $3)
        "#,
        admin_id,
        config.to_string(),
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}