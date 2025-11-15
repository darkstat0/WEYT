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
pub struct StudioStatsRequest {
    pub user_id: Uuid,
    pub time_range: String, // "7d", "30d", "90d", "1y"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoExportRequest {
    pub video_id: Uuid,
    pub format: String, // "mp4", "mov", "avi", "gif", "mp3"
    pub quality: String, // "1080p", "720p", "480p", "360p"
    pub start_time: Option<i32>,
    pub end_time: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorStats {
    pub total_videos: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_comments: i64,
    pub total_shares: i64,
    pub total_saves: i64,
    pub total_revenue: f64,
    pub avg_view_duration: f64,
    pub avg_engagement_rate: f64,
    pub top_performing_video: Option<VideoPerformance>,
    pub growth_metrics: GrowthMetrics,
    pub audience_demographics: AudienceDemographics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoPerformance {
    pub id: Uuid,
    pub title: String,
    pub views: i64,
    pub likes: i64,
    pub comments: i64,
    pub shares: i64,
    pub engagement_rate: f64,
    pub revenue: f64,
    pub publish_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrowthMetrics {
    pub views_growth: f64,
    pub subscribers_growth: f64,
    pub engagement_growth: f64,
    pub revenue_growth: f64,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudienceDemographics {
    pub age_groups: HashMap<String, f64>,
    pub gender_distribution: HashMap<String, f64>,
    pub top_countries: Vec<CountryData>,
    pub viewing_times: Vec<TimeData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryData {
    pub country: String,
    pub percentage: f64,
    pub views: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeData {
    pub hour: i32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub download_url: String,
    pub file_size: i64,
    pub format: String,
    pub processing_time: f64,
}

// Get creator studio statistics
pub async fn studio_stats(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<StudioStatsRequest>,
) -> Result<HttpResponse> {
    let stats = get_creator_stats(pool, user.id, &req.time_range).await;

    match stats {
        Ok(stats_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Studio statistics retrieved".to_string(),
                data: Some(serde_json::to_value(stats_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get creator stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get creator statistics".to_string(),
                errors: None,
            }))
        }
    }
}

// Get creator statistics
async fn get_creator_stats(
    pool: &DbPool,
    user_id: Uuid,
    time_range: &str,
) -> Result<CreatorStats, Box<dyn std::error::Error + Send + Sync>> {
    // Calculate date range
    let (start_date, end_date) = calculate_date_range(time_range);

    // Get basic statistics
    let basic_stats = get_basic_stats(pool, user_id, start_date, end_date).await?;

    // Get top performing video
    let top_video = get_top_performing_video(pool, user_id, start_date, end_date).await?;

    // Get growth metrics
    let growth_metrics = calculate_growth_metrics(pool, user_id, time_range).await?;

    // Get audience demographics
    let audience_demographics = get_audience_demographics(pool, user_id, start_date, end_date).await?;

    // Calculate averages
    let avg_view_duration = calculate_avg_view_duration(pool, user_id, start_date, end_date).await?;
    let avg_engagement_rate = calculate_avg_engagement_rate(pool, user_id, start_date, end_date).await?;

    Ok(CreatorStats {
        total_videos: basic_stats.total_videos,
        total_views: basic_stats.total_views,
        total_likes: basic_stats.total_likes,
        total_comments: basic_stats.total_comments,
        total_shares: basic_stats.total_shares,
        total_saves: basic_stats.total_saves,
        total_revenue: basic_stats.total_revenue,
        avg_view_duration,
        avg_engagement_rate,
        top_performing_video: top_video,
        growth_metrics,
        audience_demographics,
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

// Get basic statistics
async fn get_basic_stats(
    pool: &DbPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<BasicStats, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        SELECT 
            COUNT(*) as total_videos,
            COALESCE(SUM(view_count), 0) as total_views,
            COALESCE(SUM(like_count), 0) as total_likes,
            COALESCE(SUM(comment_count), 0) as total_comments,
            COALESCE(SUM(share_count), 0) as total_shares,
            COALESCE(SUM(save_count), 0) as total_saves,
            COALESCE(SUM(revenue_amount), 0) as total_revenue
        FROM videos v
        LEFT JOIN creator_revenue cr ON v.id = cr.video_id
        WHERE v.user_id = $1 
          AND v.created_at BETWEEN $2 AND $3
          AND v.status = 'ready'
    "#;

    let stats = sqlx::query_as::<_, BasicStats>(query)
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

    Ok(stats)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct BasicStats {
    pub total_videos: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_comments: i64,
    pub total_shares: i64,
    pub total_saves: i64,
    pub total_revenue: f64,
}

// Get top performing video
async fn get_top_performing_video(
    pool: &DbPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Option<VideoPerformance>, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        SELECT 
            v.id, v.title, v.view_count, v.like_count, v.comment_count, v.share_count,
            (v.like_count + v.comment_count + v.share_count) * 100.0 / NULLIF(v.view_count, 0) as engagement_rate,
            COALESCE(SUM(cr.amount), 0) as revenue,
            v.created_at as publish_date
        FROM videos v
        LEFT JOIN creator_revenue cr ON v.id = cr.video_id
        WHERE v.user_id = $1 
          AND v.created_at BETWEEN $2 AND $3
          AND v.status = 'ready'
        GROUP BY v.id, v.title, v.view_count, v.like_count, v.comment_count, v.share_count, v.created_at
        ORDER BY engagement_rate DESC, v.view_count DESC
        LIMIT 1
    "#;

    let video = sqlx::query_as::<_, VideoPerformance>(query)
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_optional(pool)
        .await?;

    Ok(video)
}

// Calculate growth metrics
async fn calculate_growth_metrics(
    pool: &DbPool,
    user_id: Uuid,
    time_range: &str,
) -> Result<GrowthMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let current_period = calculate_date_range(time_range);
    let previous_period = match time_range {
        "7d" => (current_period.0 - chrono::Duration::days(7), current_period.1 - chrono::Duration::days(7)),
        "30d" => (current_period.0 - chrono::Duration::days(30), current_period.1 - chrono::Duration::days(30)),
        "90d" => (current_period.0 - chrono::Duration::days(90), current_period.1 - chrono::Duration::days(90)),
        "1y" => (current_period.0 - chrono::Duration::days(365), current_period.1 - chrono::Duration::days(365)),
        _ => (current_period.0 - chrono::Duration::days(30), current_period.1 - chrono::Duration::days(30)),
    };

    // Get current period stats
    let current_stats = get_basic_stats(pool, user_id, current_period.0, current_period.1).await?;
    
    // Get previous period stats
    let previous_stats = get_basic_stats(pool, user_id, previous_period.0, previous_period.1).await?;

    // Calculate growth rates
    let views_growth = calculate_growth_rate(previous_stats.total_views, current_stats.total_views);
    let subscribers_growth = calculate_subscriber_growth(pool, user_id, &previous_period, &current_period).await?;
    let engagement_growth = calculate_growth_rate(
        previous_stats.total_likes + previous_stats.total_comments + previous_stats.total_shares,
        current_stats.total_likes + current_stats.total_comments + current_stats.total_shares,
    );
    let revenue_growth = calculate_growth_rate(previous_stats.total_revenue, current_stats.total_revenue);

    Ok(GrowthMetrics {
        views_growth,
        subscribers_growth,
        engagement_growth,
        revenue_growth,
        period: time_range.to_string(),
    })
}

// Calculate growth rate
fn calculate_growth_rate(previous: f64, current: f64) -> f64 {
    if previous == 0.0 {
        return if current > 0.0 { 100.0 } else { 0.0 };
    }
    ((current - previous) / previous) * 100.0
}

// Calculate subscriber growth
async fn calculate_subscriber_growth(
    pool: &DbPool,
    user_id: Uuid,
    previous_period: &(DateTime<Utc>, DateTime<Utc>),
    current_period: &(DateTime<Utc>, DateTime<Utc>),
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // Get current subscriber count
    let current_subscribers = sqlx::query!(
        "SELECT COUNT(*) as count FROM subscriptions WHERE target_user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?
    .count;

    // Get subscribers from previous period
    let previous_subscribers = sqlx::query!(
        "SELECT COUNT(*) as count FROM subscriptions WHERE target_user_id = $1 AND created_at BETWEEN $2 AND $3",
        user_id,
        previous_period.0,
        previous_period.1
    )
    .fetch_one(pool)
    .await?
    .count;

    Ok(calculate_growth_rate(previous_subscribers as f64, current_subscribers as f64))
}

// Get audience demographics
async fn get_audience_demographics(
    pool: &DbPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<AudienceDemographics, Box<dyn std::error::Error + Send + Sync>> {
    // Get age distribution (placeholder data)
    let age_groups = HashMap::from([
        ("13-17".to_string(), 0.15),
        ("18-24".to_string(), 0.35),
        ("25-34".to_string(), 0.30),
        ("35-44".to_string(), 0.15),
        ("45+".to_string(), 0.05),
    ]);

    // Get gender distribution (placeholder data)
    let gender_distribution = HashMap::from([
        ("male".to_string(), 0.45),
        ("female".to_string(), 0.50),
        ("other".to_string(), 0.05),
    ]);

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
            COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percentage,
            COUNT(*) as views
        FROM video_views vv
        JOIN videos v ON vv.video_id = v.id
        WHERE v.user_id = $1 AND vv.created_at BETWEEN $2 AND $3
        GROUP BY country
        ORDER BY views DESC
        LIMIT 5
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    // Get viewing times
    let viewing_times = sqlx::query_as!(
        TimeData,
        r#"
        SELECT 
            EXTRACT(HOUR FROM created_at) as hour,
            COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percentage
        FROM video_views vv
        JOIN videos v ON vv.video_id = v.id
        WHERE v.user_id = $1 AND vv.created_at BETWEEN $2 AND $3
        GROUP BY hour
        ORDER BY hour
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(AudienceDemographics {
        age_groups,
        gender_distribution,
        top_countries,
        viewing_times,
    })
}

// Calculate average view duration
async fn calculate_avg_view_duration(
    pool: &DbPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query!(
        r#"
        SELECT AVG(watch_duration) as avg_duration
        FROM video_views vv
        JOIN videos v ON vv.video_id = v.id
        WHERE v.user_id = $1 AND vv.created_at BETWEEN $2 AND $3
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    Ok(result.avg_duration.unwrap_or(0.0))
}

// Calculate average engagement rate
async fn calculate_avg_engagement_rate(
    pool: &DbPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query!(
        r#"
        SELECT AVG((like_count + comment_count + share_count) * 100.0 / NULLIF(view_count, 0)) as avg_rate
        FROM videos
        WHERE user_id = $1 AND created_at BETWEEN $2 AND $3 AND status = 'ready'
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    Ok(result.avg_rate.unwrap_or(0.0))
}

// Export video
pub async fn export_video(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<VideoExportRequest>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    // Verify user owns the video
    let video = sqlx::query!(
        "SELECT id, video_url, duration FROM videos WHERE id = $1 AND user_id = $2",
        req.video_id,
        user.id
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let video = match video {
        Some(video) => video,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Video not found or access denied".to_string(),
                errors: None,
            }));
        }
    };

    // Validate export parameters
    if !is_valid_export_format(&req.format) {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Invalid export format".to_string(),
            errors: None,
        }));
    }

    if !is_valid_export_quality(&req.quality) {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Invalid export quality".to_string(),
            errors: None,
        }));
    }

    // Validate time range
    if let Some(start) = req.start_time {
        if start < 0 || start >= video.duration {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Invalid start time".to_string(),
                errors: None,
            }));
        }
    }

    if let Some(end) = req.end_time {
        if end <= 0 || end > video.duration {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Invalid end time".to_string(),
                errors: None,
            }));
        }
    }

    // Start export process
    let export_result = start_video_export(
        &video.video_url,
        &req.format,
        &req.quality,
        req.start_time,
        req.end_time,
        &config,
    )
    .await;

    match export_result {
        Ok(result) => {
            // Record export in database
            if let Err(e) = record_export(pool, user.id, req.video_id, &req.format, &result).await {
                eprintln!("Failed to record export: {}", e);
            }

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Video export started".to_string(),
                data: Some(serde_json::to_value(result).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to export video".to_string(),
                errors: None,
            }))
        }
    }
}

// Check if export format is valid
fn is_valid_export_format(format: &str) -> bool {
    matches!(format, "mp4" | "mov" | "avi" | "gif" | "mp3" | "wav")
}

// Check if export quality is valid
fn is_valid_export_quality(quality: &str) -> bool {
    matches!(quality, "1080p" | "720p" | "480p" | "360p" | "240p")
}

// Start video export process
async fn start_video_export(
    video_url: &str,
    format: &str,
    quality: &str,
    start_time: Option<i32>,
    end_time: Option<i32>,
    config: &Config,
) -> Result<ExportResult, Box<dyn std::error::Error + Send + Sync>> {
    // In production, use FFmpeg or similar tool to export video
    // For now, return placeholder result
    let download_url = format!("{}/exports/{}.{}", config.s3.public_url, Uuid::new_v4(), format);
    
    Ok(ExportResult {
        download_url,
        file_size: 1024 * 1024 * 10, // 10MB placeholder
        format: format.to_string(),
        processing_time: 30.0, // 30 seconds placeholder
    })
}

// Record export in database
async fn record_export(
    pool: &DbPool,
    user_id: Uuid,
    video_id: Uuid,
    format: &str,
    result: &ExportResult,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO video_exports (user_id, video_id, format, download_url, file_size, processing_time)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        video_id,
        format,
        result.download_url,
        result.file_size,
        result.processing_time
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Get trending topics for creator
pub async fn get_trending_topics(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    let topics = get_creator_trending_topics(pool, user.id).await;

    match topics {
        Ok(topics_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Trending topics retrieved".to_string(),
                data: Some(serde_json::to_value(topics_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get trending topics: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get trending topics".to_string(),
                errors: None,
            }))
        }
    }
}

// Get trending topics for creator
async fn get_creator_trending_topics(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<TrendingTopic>, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        SELECT 
            topic,
            COUNT(*) as video_count,
            AVG(view_count) as avg_views,
            AVG(engagement_rate) as avg_engagement,
            trend_score
        FROM (
            SELECT 
                v.topic_category as topic,
                v.view_count,
                (v.like_count + v.comment_count + v.share_count) * 100.0 / NULLIF(v.view_count, 0) as engagement_rate,
                v.view_count * (v.like_count + v.comment_count + v.share_count) as trend_score
            FROM videos v
            WHERE v.user_id = $1 AND v.status = 'ready' AND v.topic_category IS NOT NULL
        ) as topic_stats
        GROUP BY topic, trend_score
        ORDER BY trend_score DESC
        LIMIT 10
    "#;

    let topics = sqlx::query_as::<_, TrendingTopic>(query)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    Ok(topics)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct TrendingTopic {
    pub topic: String,
    pub video_count: i64,
    pub avg_views: f64,
    pub avg_engagement: f64,
    pub trend_score: f64,
}

// Get content suggestions
pub async fn get_content_suggestions(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    let suggestions = get_content_ideas(pool, user.id).await;

    match suggestions {
        Ok(suggestions_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Content suggestions retrieved".to_string(),
                data: Some(serde_json::to_value(suggestions_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get content suggestions: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get content suggestions".to_string(),
                errors: None,
            }))
        }
    }
}

// Get content ideas for creator
async fn get_content_ideas(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<ContentIdea>, Box<dyn std::error::Error + Send + Sync>> {
    // Get creator's existing content topics
    let existing_topics = sqlx::query!(
        "SELECT DISTINCT topic_category FROM videos WHERE user_id = $1 AND topic_category IS NOT NULL",
        user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| row.topic_category.unwrap_or_default())
    .collect::<Vec<String>>();

    // Get trending topics globally
    let trending_topics = sqlx::query_as!(
        TrendingTopic,
        r#"
        SELECT 
            topic_category as topic,
            COUNT(*) as video_count,
            AVG(view_count) as avg_views,
            AVG((like_count + comment_count + share_count) * 100.0 / NULLIF(view_count, 0)) as avg_engagement,
            view_count * (like_count + comment_count + share_count) as trend_score
        FROM videos
        WHERE status = 'ready' AND topic_category IS NOT NULL
        GROUP BY topic_category, trend_score
        ORDER BY trend_score DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;

    // Filter out existing topics and get suggestions
    let suggestions: Vec<ContentIdea> = trending_topics
        .into_iter()
        .filter(|topic| !existing_topics.contains(&topic.topic))
        .take(10)
        .map(|topic| ContentIdea {
            topic: topic.topic,
            estimated_views: topic.avg_views as i64,
            estimated_engagement: topic.avg_engagement,
            competition_level: calculate_competition_level(topic.video_count),
            trending_score: topic.trend_score,
        })
        .collect();

    Ok(suggestions)
}

// Calculate competition level
fn calculate_competition_level(video_count: i64) -> String {
    if video_count < 1000 {
        "Low".to_string()
    } else if video_count < 10000 {
        "Medium".to_string()
    } else {
        "High".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentIdea {
    pub topic: String,
    pub estimated_views: i64,
    pub estimated_engagement: f64,
    pub competition_level: String,
    pub trending_score: f64,
}

// Get A/B test results for thumbnails
pub async fn get_ab_test_results(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    video_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let results = get_thumbnail_ab_test_results(pool, user.id, video_id.into_inner()).await;

    match results {
        Ok(results_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "A/B test results retrieved".to_string(),
                data: Some(serde_json::to_value(results_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get A/B test results: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get A/B test results".to_string(),
                errors: None,
            }))
        }
    }
}

// Get A/B test results for thumbnails
async fn get_thumbnail_ab_test_results(
    pool: &DbPool,
    user_id: Uuid,
    video_id: Uuid,
) -> Result<Vec<ThumbnailTestResult>, Box<dyn std::error::Error + Send + Sync>> {
    // In production, this would query A/B test results
    // For now, return placeholder data
    Ok(vec
![
        ThumbnailTestResult {
            thumbnail_id: "thumbnail_1".to_string(),
            click_through_rate: 0.05,
            views: 10000,
            engagement_rate: 0.08,
            confidence: 0.95,
            is_winner: true,
        },
        ThumbnailTestResult {
            thumbnail_id: "thumbnail_2".to_string(),
            click_through_rate: 0.03,
            views: 10000,
            engagement_rate: 0.06,
            confidence: 0.95,
            is_winner: false,
        },
    ])
}

#[derive(Debug, Serialize, Deserialize)]
struct ThumbnailTestResult {
    pub thumbnail_id: String,
    pub click_through_rate: f64,
    pub views: i64,
    pub engagement_rate: f64,
    pub confidence: f64,
    pub is_winner: bool,
}