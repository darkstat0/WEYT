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
pub struct GetRevenueRequest {
    pub creator_id: Uuid,
    pub time_range: String, // "7d", "30d", "90d", "1y"
    pub revenue_type: Option<String>, // "ads", "premium", "membership", "tips", "superchat", "brand_deal"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueBreakdown {
    pub total_revenue: f64,
    pub by_type: HashMap<String, f64>,
    pub by_date: Vec<DailyRevenue>,
    pub top_videos: Vec<VideoRevenue>,
    pub growth_metrics: RevenueGrowth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyRevenue {
    pub date: String,
    pub revenue: f64,
    pub views: i64,
    pub engagements: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoRevenue {
    pub video_id: Uuid,
    pub title: String,
    pub revenue: f64,
    pub views: i64,
    pub rpm: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueGrowth {
    pub revenue_growth: f64,
    pub views_growth: f64,
    pub rpm_growth: f64,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdCampaignRequest {
    pub name: String,
    pub budget: f64,
    pub target_audience: serde_json::Value,
    pub ad_formats: Vec<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub bid_strategy: String, // "cpm", "cpc", "cpa"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdCampaign {
    pub id: Uuid,
    pub name: String,
    pub budget: f64,
    pub spent: f64,
    pub status: String,
    pub target_audience: serde_json::Value,
    pub ad_formats: Vec<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub bid_strategy: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrandDealRequest {
    pub title: String,
    pub description: String,
    pub compensation: f64,
    pub deliverables: Vec<String>,
    pub requirements: serde_json::Value,
    pub deadline: DateTime<Utc>,
    pub target_audience: serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrandDeal {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub brand_id: Uuid,
    pub title: String,
    pub description: String,
    pub compensation: f64,
    pub deliverables: Vec<String>,
    pub requirements: serde_json::Value>,
    pub status: String,
    pub deadline: DateTime<Utc>,
    pub target_audience: serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Get creator revenue breakdown
pub async fn get_revenue(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<GetRevenueRequest>,
) -> Result<HttpResponse> {
    // Check if user is creator or admin
    if user.role != "creator" && user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let creator_id = if user.role == "admin" {
        req.creator_id
    } else {
        user.id
    };

    let breakdown = get_revenue_breakdown(pool, creator_id, &req.time_range, req.revenue_type.as_deref()).await;

    match breakdown {
        Ok(breakdown_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Revenue breakdown retrieved".to_string(),
                data: Some(serde_json::to_value(breakdown_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get revenue breakdown: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get revenue data".to_string(),
                errors: None,
            }))
        }
    }
}

// Get revenue breakdown
async fn get_revenue_breakdown(
    pool: &DbPool,
    creator_id: Uuid,
    time_range: &str,
    revenue_type: Option<&str>,
) -> Result<RevenueBreakdown, Box<dyn std::error::Error + Send + Sync>> {
    // Calculate date range
    let (start_date, end_date) = calculate_date_range(time_range);

    // Get total revenue
    let total_revenue = get_total_revenue(pool, creator_id, start_date, end_date, revenue_type).await?;

    // Get revenue by type
    let by_type = get_revenue_by_type(pool, creator_id, start_date, end_date, revenue_type).await?;

    // Get revenue by date
    let by_date = get_revenue_by_date(pool, creator_id, start_date, end_date, revenue_type).await?;

    // Get top performing videos
    let top_videos = get_top_revenue_videos(pool, creator_id, start_date, end_date, revenue_type).await?;

    // Calculate growth metrics
    let growth_metrics = calculate_revenue_growth(pool, creator_id, time_range).await?;

    Ok(RevenueBreakdown {
        total_revenue,
        by_type,
        by_date,
        top_videos,
        growth_metrics,
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

// Get total revenue
async fn get_total_revenue(
    pool: &DbPool,
    creator_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    revenue_type: Option<&str>,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = String::from(
        r#"
        SELECT COALESCE(SUM(amount), 0) as total
        FROM creator_revenue
        WHERE creator_id = $1 AND status = 'paid'
          AND created_at BETWEEN $2 AND $3
        "#,
    );

    let mut query_params = vec
![Box::new(creator_id), Box::new(start_date), Box::new(end_date)];

    if let Some(rev_type) = revenue_type {
        query.push_str(&format!(" AND revenue_type = ${}", query_params.len() + 1));
        query_params.push(Box::new(rev_type.to_string()));
    }

    let result = sqlx::query!(query.as_str())
        .bind_all(query_params)
        .fetch_one(pool)
        .await?;

    Ok(result.total.unwrap_or(0.0))
}

// Get revenue by type
async fn get_revenue_by_type(
    pool: &DbPool,
    creator_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    revenue_type: Option<&str>,
) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = String::from(
        r#"
        SELECT revenue_type, COALESCE(SUM(amount), 0) as total
        FROM creator_revenue
        WHERE creator_id = $1 AND status = 'paid'
          AND created_at BETWEEN $2 AND $3
        "#,
    );

    let mut query_params = vec
![Box::new(creator_id), Box::new(start_date), Box::new(end_date)];

    if let Some(rev_type) = revenue_type {
        query.push_str(&format!(" AND revenue_type = ${}", query_params.len() + 1));
        query_params.push(Box::new(rev_type.to_string()));
    }

    query.push_str(" GROUP BY revenue_type");

    let results = sqlx::query!(query.as_str())
        .bind_all(query_params)
        .fetch_all(pool)
        .await?;

    let mut by_type = HashMap::new();
    for result in results {
        by_type.insert(result.revenue_type, result.total.unwrap_or(0.0));
    }

    Ok(by_type)
}

// Get revenue by date
async fn get_revenue_by_date(
    pool: &DbPool,
    creator_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    revenue_type: Option<&str>,
) -> Result<Vec<DailyRevenue>, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = String::from(
        r#"
        SELECT 
            DATE(created_at) as date,
            COALESCE(SUM(amount), 0) as revenue,
            COALESCE(SUM(v.view_count), 0) as views,
            COALESCE(SUM(v.like_count + v.comment_count + v.share_count), 0) as engagements
        FROM creator_revenue cr
        LEFT JOIN videos v ON cr.video_id = v.id
        WHERE cr.creator_id = $1 AND cr.status = 'paid'
          AND cr.created_at BETWEEN $2 AND $3
        "#,
    );

    let mut query_params = vec
![Box::new(creator_id), Box::new(start_date), Box::new(end_date)];

    if let Some(rev_type) = revenue_type {
        query.push_str(&format!(" AND cr.revenue_type = ${}", query_params.len() + 1));
        query_params.push(Box::new(rev_type.to_string()));
    }

    query.push_str(
        r#"
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
    );

    let results = sqlx::query!(query.as_str())
        .bind_all(query_params)
        .fetch_all(pool)
        .await?;

    let by_date: Vec<DailyRevenue> = results
        .into_iter()
        .map(|row| DailyRevenue {
            date: row.date.format("%Y-%m-%d").to_string(),
            revenue: row.revenue.unwrap_or(0.0),
            views: row.views.unwrap_or(0),
            engagements: row.engagements.unwrap_or(0),
        })
        .collect();

    Ok(by_date)
}

// Get top revenue videos
async fn get_top_revenue_videos(
    pool: &DbPool,
    creator_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    revenue_type: Option<&str>,
) -> Result<Vec<VideoRevenue>, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = String::from(
        r#"
        SELECT 
            v.id, v.title,
            COALESCE(SUM(cr.amount), 0) as revenue,
            COALESCE(v.view_count, 0) as views,
            CASE WHEN v.view_count > 0 THEN COALESCE(SUM(cr.amount), 0) * 1000.0 / v.view_count ELSE 0 END as rpm
        FROM videos v
        LEFT JOIN creator_revenue cr ON v.id = cr.video_id
        WHERE v.user_id = $1 AND v.status = 'ready'
          AND (cr.created_at BETWEEN $2 AND $3 OR cr.created_at IS NULL)
        "#,
    );

    let mut query_params = vec
![Box::new(creator_id), Box::new(start_date), Box::new(end_date)];

    if let Some(rev_type) = revenue_type {
        query.push_str(&format!(" AND cr.revenue_type = ${}", query_params.len() + 1));
        query_params.push(Box::new(rev_type.to_string()));
    }

    query.push_str(
        r#"
        GROUP BY v.id, v.title, v.view_count
        ORDER BY revenue DESC
        LIMIT 10
        "#,
    );

    let results = sqlx::query!(query.as_str())
        .bind_all(query_params)
        .fetch_all(pool)
        .await?;

    let top_videos: Vec<VideoRevenue> = results
        .into_iter()
        .map(|row| VideoRevenue {
            video_id: row.id,
            title: row.title,
            revenue: row.revenue.unwrap_or(0.0),
            views: row.views.unwrap_or(0),
            rpm: row.rpm.unwrap_or(0.0),
        })
        .collect();

    Ok(top_videos)
}

// Calculate revenue growth
async fn calculate_revenue_growth(
    pool: &DbPool,
    creator_id: Uuid,
    time_range: &str,
) -> Result<RevenueGrowth, Box<dyn std::error::Error + Send + Sync>> {
    let current_period = calculate_date_range(time_range);
    let previous_period = match time_range {
        "7d" => (current_period.0 - chrono::Duration::days(7), current_period.1 - chrono::Duration::days(7)),
        "30d" => (current_period.0 - chrono::Duration::days(30), current_period.1 - chrono::Duration::days(30)),
        "90d" => (current_period.0 - chrono::Duration::days(90), current_period.1 - chrono::Duration::days(90)),
        "1y" => (current_period.0 - chrono::Duration::days(365), current_period.1 - chrono::Duration::days(365)),
        _ => (current_period.0 - chrono::Duration::days(30), current_period.1 - chrono::Duration::days(30)),
    };

    // Get current period revenue and views
    let (current_revenue, current_views) = get_period_metrics(pool, creator_id, &current_period).await?;
    
    // Get previous period revenue and views
    let (previous_revenue, previous_views) = get_period_metrics(pool, creator_id, &previous_period).await?;

    // Calculate growth rates
    let revenue_growth = calculate_growth_rate(previous_revenue, current_revenue);
    let views_growth = calculate_growth_rate(previous_views, current_views);
    
    // Calculate RPM (Revenue Per Mille)
    let current_rpm = if current_views > 0 { current_revenue * 1000.0 / current_views as f64 } else { 0.0 };
    let previous_rpm = if previous_views > 0 { previous_revenue * 1000.0 / previous_views as f64 } else { 0.0 };
    let rpm_growth = calculate_growth_rate(previous_rpm, current_rpm);

    Ok(RevenueGrowth {
        revenue_growth,
        views_growth,
        rpm_growth,
        period: time_range.to_string(),
    })
}

// Get period metrics
async fn get_period_metrics(
    pool: &DbPool,
    creator_id: Uuid,
    period: &(DateTime<Utc>, DateTime<Utc>),
) -> Result<(f64, i64), Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query!(
        r#"
        SELECT 
            COALESCE(SUM(amount), 0) as revenue,
            COALESCE(SUM(v.view_count), 0) as views
        FROM creator_revenue cr
        LEFT JOIN videos v ON cr.video_id = v.id
        WHERE cr.creator_id = $1 AND cr.status = 'paid'
          AND cr.created_at BETWEEN $2 AND $3
        "#,
        creator_id,
        period.0,
        period.1
    )
    .fetch_one(pool)
    .await?;

    Ok((result.revenue.unwrap_or(0.0), result.views.unwrap_or(0)))
}

// Calculate growth rate
fn calculate_growth_rate(previous: f64, current: f64) -> f64 {
    if previous == 0.0 {
        return if current > 0.0 { 100.0 } else { 0.0 };
    }
    ((current - previous) / previous) * 100.0
}

// Create ad campaign
pub async fn create_ad_campaign(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<CreateAdCampaignRequest>,
) -> Result<HttpResponse> {
    // Check if user is advertiser or admin
    if user.role != "advertiser" && user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    // Validate campaign data
    if req.start_date >= req.end_date {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "End date must be after start date".to_string(),
            errors: None,
        }));
    }

    if req.budget <= 0.0 {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Budget must be positive".to_string(),
            errors: None,
        }));
    }

    // Create campaign
    let campaign_id = Uuid::new_v4();
    let now = Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO ad_campaigns (
            id, advertiser_id, name, budget, spent, status, 
            target_audience, ad_formats, start_date, end_date, 
            bid_strategy, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
        campaign_id,
        user.id,
        req.name,
        req.budget,
        0.0, // Initial spent amount
        "active",
        req.target_audience,
        serde_json::to_string(&req.ad_formats).unwrap_or_default(),
        req.start_date,
        req.end_date,
        req.bid_strategy,
        now
    )
    .fetch_one(pool.as_ref())
    .await {
        Ok(_) => {
            Ok(HttpResponse::Created().json(utils::ApiResponse {
                success: true,
                message: "Ad campaign created successfully".to_string(),
                data: Some(serde_json::json!({"campaign_id": campaign_id})),
            }))
        }
        Err(e) => {
            eprintln!("Failed to create ad campaign: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to create ad campaign".to_string(),
                errors: None,
            }))
        }
    }
}

// Get ad campaigns
pub async fn get_ad_campaigns(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    // Check if user is advertiser or admin
    if user.role != "advertiser" && user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let campaigns = sqlx::query_as!(
        AdCampaign,
        r#"
        SELECT 
            id, advertiser_id, name, budget, spent, status,
            target_audience, ad_formats, start_date, end_date,
            bid_strategy, created_at
        FROM ad_campaigns
        WHERE advertiser_id = $1
        ORDER BY created_at DESC
        "#,
        user.id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Ad campaigns retrieved".to_string(),
        data: Some(serde_json::to_value(campaigns).unwrap()),
    }))
}

// Create brand deal
pub async fn create_brand_deal(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<BrandDealRequest>,
) -> Result<HttpResponse> {
    // Check if user is admin or brand
    if user.role != "brand" && user.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    // Validate deal data
    if req.deadline <= Utc::now() {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Deadline must be in the future".to_string(),
            errors: None,
        }));
    }

    if req.compensation <= 0.0 {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Compensation must be positive".to_string(),
            errors: None,
        }));
    }

    // Create brand deal
    let deal_id = Uuid::new_v4();
    let brand_id = if user.role == "admin" {
        // In production, get brand ID from request
        Uuid::new_v4()
    } else {
        user.id
    };

    match sqlx::query!(
        r#"
        INSERT INTO brand_deals (
            id, brand_id, title, description, compensation,
            deliverables, requirements, status, deadline,
            target_audience, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
        "#,
        deal_id,
        brand_id,
        req.title,
        req.description,
        req.compensation,
        serde_json::to_string(&req.deliverables).unwrap_or_default(),
        req.requirements,
        "pending",
        req.deadline,
        req.target_audience,
        Utc::now()
    )
    .fetch_one(pool.as_ref())
    .await {
        Ok(_) => {
            // Notify creators about new brand deal
            if let Err(e) = notify_creators_about_deal(pool, deal_id, &req.target_audience).await {
                eprintln!("Failed to notify creators: {}", e);
            }

            Ok(HttpResponse::Created().json(utils::ApiResponse {
                success: true,
                message: "Brand deal created successfully".to_string(),
                data: Some(serde_json::json!({"deal_id": deal_id})),
            }))
        }
        Err(e) => {
            eprintln!("Failed to create brand deal: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to create brand deal".to_string(),
                errors: None,
            }))
        }
    }
}

// Notify creators about brand deal
async fn notify_creators_about_deal(
    pool: &DbPool,
    deal_id: Uuid,
    target_audience: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In production, this would match creators with the target audience
    // and send notifications
    // For now, just log the action
    println!("Notifying creators about brand deal: {}", deal_id);
    
    // Record deal notification
    sqlx::query!(
        r#"
        INSERT INTO brand_deal_notifications (deal_id, created_at)
        VALUES ($1, $2)
        "#,
        deal_id,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Get available brand deals
pub async fn get_available_brand_deals(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    // Check if user is creator
    if user.role != "creator" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let deals = sqlx::query_as!(
        BrandDeal,
        r#"
        SELECT 
            id, brand_id, title, description, compensation,
            deliverables, requirements, status, deadline,
            target_audience, created_at
        FROM brand_deals
        WHERE status = 'pending'
          AND deadline > NOW()
        ORDER BY created_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Available brand deals retrieved".to_string(),
        data: Some(serde_json::to_value(deals).unwrap()),
    }))
}

// Accept brand deal
pub async fn accept_brand_deal(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    deal_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // Check if user is creator
    if user.role != "creator" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    // Check if deal exists and is pending
    let deal = sqlx::query!(
        "SELECT id, status FROM brand_deals WHERE id = $1",
        deal_id.into_inner()
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let deal = match deal {
        Some(deal) => deal,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Brand deal not found".to_string(),
                errors: None,
            }));
        }
    };

    if deal.status != "pending" {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Deal is not available".to_string(),
            errors: None,
        }));
    }

    // Update deal status
    match sqlx::query!(
        "UPDATE brand_deals SET status = 'accepted', creator_id = $1 WHERE id = $2",
        user.id,
        deal.id
    )
    .execute(pool.as_ref())
    .await {
        Ok(_) => {
            // Notify brand about acceptance
            if let Err(e) = notify_brand_acceptance(pool, deal.id, user.id).await {
                eprintln!("Failed to notify brand: {}", e);
            }

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Brand deal accepted successfully".to_string(),
                data: None,
            }))
        }
        Err(e) => {
            eprintln!("Failed to accept brand deal: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to accept brand deal".to_string(),
                errors: None,
            }))
        }
    }
}

// Notify brand about acceptance
async fn notify_brand_acceptance(
    pool: &DbPool,
    deal_id: Uuid,
    creator_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get brand ID from deal
    let brand_id = sqlx::query!(
        "SELECT brand_id FROM brand_deals WHERE id = $1",
        deal_id
    )
    .fetch_one(pool)
    .await?
    .brand_id;

    // Record acceptance notification
    sqlx::query!(
        r#"
        INSERT INTO brand_deal_acceptances (deal_id, brand_id, creator_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        deal_id,
        brand_id,
        creator_id,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Get payment history
pub async fn get_payment_history(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    let payments = sqlx::query!(
        r#"
        SELECT 
            id, amount, currency, status, created_at, paid_at,
            revenue_type, video_id, description
        FROM creator_revenue
        WHERE creator_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        user.id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Payment history retrieved".to_string(),
        data: Some(serde_json::to_value(payments).unwrap()),
    }))
}

// Set up payout method
pub async fn setup_payout_method(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<SetupPayoutRequest>,
) -> Result<HttpResponse> {
    // Validate payout method
    if !is_valid_payout_method(&req.method) {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Invalid payout method".to_string(),
            errors: None,
        }));
    }

    // Update user payout information
    match sqlx::query!(
        r#"
        UPDATE users 
        SET payout_method = $1, payout_details = $2, updated_at = $3
        WHERE id = $4
        "#,
        req.method,
        serde_json::to_value(&req.details).unwrap_or_default(),
        Utc::now(),
        user.id
    )
    .execute(pool.as_ref())
    .await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Payout method updated successfully".to_string(),
                data: None,
            }))
        }
        Err(e) => {
            eprintln!("Failed to update payout method: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to update payout method".to_string(),
                errors: None,
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupPayoutRequest {
    pub method: String, // "paypal", "stripe", "bank_transfer"
    pub details: serde_json::Value,
}

// Check if payout method is valid
fn is_valid_payout_method(method: &str) -> bool {
    matches!(method, "paypal" | "stripe" | "bank_transfer")
}

// Request payout
pub async fn request_payout(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    amount: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let payout_amount = amount.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);

    if payout_amount <= 0.0 {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Invalid payout amount".to_string(),
            errors: None,
        }));
    }

    // Get user's available balance
    let available_balance = get_available_balance(pool, user.id).await?;

    if payout_amount > available_balance {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient balance".to_string(),
            errors: None,
        }));
    }

    // Create payout request
    let payout_id = Uuid::new_v4();
    let now = Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO payout_requests (
            id, creator_id, amount, currency, status, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        payout_id,
        user.id,
        payout_amount,
        "USD",
        "pending",
        now
    )
    .fetch_one(pool.as_ref())
    .await {
        Ok(_) => {
            // Update available balance
            if let Err(e) = update_available_balance(pool, user.id, payout_amount).await {
                eprintln!("Failed to update balance: {}", e);
            }

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Payout request submitted successfully".to_string(),
                data: Some(serde_json::json!({"payout_id": payout_id})),
            }))
        }
        Err(e) => {
            eprintln!("Failed to create payout request: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to request payout".to_string(),
                errors: None,
            }))
        }
    }
}

// Get available balance
async fn get_available_balance(pool: &DbPool, creator_id: Uuid) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(amount), 0) as balance
        FROM creator_revenue
        WHERE creator_id = $1 AND status = 'paid' AND paid_at IS NULL
        "#,
        creator_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.balance.unwrap_or(0.0))
}

// Update available balance
async fn update_available_balance(
    pool: &DbPool,
    creator_id: Uuid,
    amount: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        UPDATE creator_revenue 
        SET status = 'processing', updated_at = $1
        WHERE creator_id = $2 AND status = 'paid' AND paid_at IS NULL
          AND amount <= $3
        ORDER BY created_at ASC
        LIMIT 1
        "#,
        Utc::now(),
        creator_id,
        amount
    )
    .execute(pool)
    .await?;

    Ok(())
}