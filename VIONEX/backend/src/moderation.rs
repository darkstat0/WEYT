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
pub struct ModerateContentRequest {
    pub content_id: Uuid,
    pub content_type: String, // "video", "comment", "user"
    pub action: String, // "approve", "reject", "flag", "delete"
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationQueueItem {
    pub id: Uuid,
    pub content_type: String,
    pub content_id: Uuid,
    pub reported_by: Uuid,
    pub reason: Option<String>,
    pub priority: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub content_preview: Option<String>,
    pub ai_analysis: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationStats {
    pub total_pending: i64,
    pub total_reviewed: i64,
    pub total_approved: i64,
    pub total_rejected: i64,
    pub avg_review_time: f64,
    pub top_violation_types: Vec<ViolationType>,
    pub moderator_performance: Vec<ModeratorPerformance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViolationType {
    pub violation_type: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeratorPerformance {
    pub moderator_id: Uuid,
    pub moderator_name: String,
    pub items_reviewed: i64,
    pub accuracy_rate: f64,
    pub avg_review_time: f64,
    pub top_violations_handled: Vec<String>,
}

// Moderate content
pub async fn moderate_content(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<ModerateContentRequest>,
) -> Result<HttpResponse> {
    // Check if user has moderation permissions
    if !has_moderation_permissions(&user) {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    // Get queue item
    let queue_item = sqlx::query_as!(
        ModerationQueueItem,
        r#"
        SELECT id, content_type, content_id, reported_by, reason, priority, status, created_at
        FROM moderation_queue
        WHERE id = $1 AND status = 'pending'
        "#,
        req.content_id
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let queue_item = match queue_item {
        Some(item) => item,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Content not found in moderation queue".to_string(),
                errors: None,
            }));
        }
    };

    // Perform moderation action
    match req.action.as_str() {
        "approve" => {
            if let Err(e) = approve_content(pool, &queue_item, &req.reason, &req.notes).await {
                eprintln!("Failed to approve content: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to approve content".to_string(),
                    errors: None,
                }));
            }
        }
        "reject" => {
            if let Err(e) = reject_content(pool, &queue_item, &req.reason, &req.notes).await {
                eprintln!("Failed to reject content: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to reject content".to_string(),
                    errors: None,
                }));
            }
        }
        "delete" => {
            if let Err(e) = delete_content(pool, &queue_item, &req.reason, &req.notes).await {
                eprintln!("Failed to delete content: {}", e);
                return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to delete content".to_string(),
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

    // Update queue item
    if let Err(e) = update_queue_item(pool, &queue_item, user.id, &req.action, &req.notes).await {
        eprintln!("Failed to update queue item: {}", e);
    }

    // Record moderation action
    if let Err(e) = record_moderation_action(pool, user.id, &queue_item, &req.action, &req.reason).await {
        eprintln!("Failed to record moderation action: {}", e);
    }

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Content moderated successfully".to_string(),
        data: None,
    }))
}

// Check if user has moderation permissions
fn has_moderation_permissions(user: &crate::auth::User) -> bool {
    matches!(user.role.as_str(), "moderator" | "admin")
}

// Approve content
async fn approve_content(
    pool: &DbPool,
    queue_item: &ModerationQueueItem,
    reason: &Option<String>,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match queue_item.content_type.as_str() {
        "video" => {
            sqlx::query!(
                "UPDATE videos SET status = 'ready' WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        "comment" => {
            sqlx::query!(
                "UPDATE comments SET is_deleted = FALSE WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        _ => {
            return Err("Unsupported content type".into());
        }
    }

    Ok(())
}

// Reject content
async fn reject_content(
    pool: &DbPool,
    queue_item: &ModerationQueueItem,
    reason: &Option<String>,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match queue_item.content_type.as_str() {
        "video" => {
            sqlx::query!(
                "UPDATE videos SET status = 'rejected' WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        "comment" => {
            sqlx::query!(
                "UPDATE comments SET is_deleted = TRUE WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        _ => {
            return Err("Unsupported content type".into());
        }
    }

    Ok(())
}

// Delete content
async fn delete_content(
    pool: &DbPool,
    queue_item: &ModerationQueueItem,
    reason: &Option<String>,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match queue_item.content_type.as_str() {
        "video" => {
            sqlx::query!(
                "UPDATE videos SET status = 'deleted' WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        "comment" => {
            sqlx::query!(
                "UPDATE comments SET is_deleted = TRUE WHERE id = $1",
                queue_item.content_id
            )
            .execute(pool)
            .await?;
        }
        _ => {
            return Err("Unsupported content type".into());
        }
    }

    Ok(())
}

// Update queue item
async fn update_queue_item(
    pool: &DbPool,
    queue_item: &ModerationQueueItem,
    moderator_id: Uuid,
    action: &str,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        UPDATE moderation_queue 
        SET status = $1, moderator_id = $2, decision = $3, notes = $4, reviewed_at = $5
        WHERE id = $6
        "#,
        format!("{}_{}", action, "ed"), // approved, rejected, deleted
        moderator_id,
        action,
        notes,
        now,
        queue_item.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Record moderation action
async fn record_moderation_action(
    pool: &DbPool,
    moderator_id: Uuid,
    queue_item: &ModerationQueueItem,
    action: &str,
    reason: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO moderation_actions (moderator_id, content_type, content_id, action, reason, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        moderator_id,
        queue_item.content_type,
        queue_item.content_id,
        action,
        reason,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Get moderation queue
pub async fn get_moderation_queue(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    params: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    // Check if user has moderation permissions
    if !has_moderation_permissions(&user) {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let content_type = params.get("content_type").and_then(|v| v.as_str());
    let status = params.get("status").and_then(|v| v.as_str());
    let priority = params.get("priority").and_then(|v| v.as_i64());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as i32;
    let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as i32;

    let mut query = String::from(
        r#"
        SELECT 
            mq.id, mq.content_type, mq.content_id, mq.reported_by, mq.reason, mq.priority, mq.status, mq.created_at,
            u.username as reported_by_name,
            v.title as video_title,
            c.content as comment_content,
            u2.username as content_owner
        FROM moderation_queue mq
        LEFT JOIN users u ON mq.reported_by = u.id
        LEFT JOIN videos v ON mq.content_type = 'video' AND mq.content_id = v.id
        LEFT JOIN comments c ON mq.content_type = 'comment' AND mq.content_id = c.id
        LEFT JOIN users u2 ON mq.content_type IN ('video', 'comment') AND 
              (mq.content_type = 'video' AND v.user_id = u2.id OR 
               mq.content_type = 'comment' AND c.user_id = u2.id)
        WHERE mq.status = 'pending'
        "#,
    );

    let mut query_params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![];

    // Add filters
    if let Some(content_type) = content_type {
        query.push_str(&format!(" AND mq.content_type = ${}", query_params.len() + 1));
        query_params.push(Box::new(content_type.to_string()));
    }

    if let Some(status) = status {
        query.push_str(&format!(" AND mq.status = ${}", query_params.len() + 1));
        query_params.push(Box::new(status.to_string()));
    }

    if let Some(priority) = priority {
        query.push_str(&format!(" AND mq.priority = ${}", query_params.len() + 1));
        query_params.push(Box::new(priority as i32));
    }

    // Add ordering
    query.push_str(" ORDER BY mq.priority DESC, mq.created_at ASC LIMIT $");
    query.push_str(&(query_params.len() + 2).to_string());
    query.push_str(" OFFSET $");
    query.push_str(&(query_params.len() + 3).to_string());

    query_params.push(Box::new(limit));
    query_params.push(Box::new(offset));

    let queue_items = sqlx::query_as::<_, ModerationQueueItemWithDetails>(query.as_str())
        .bind_all(query_params)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Moderation queue retrieved".to_string(),
        data: Some(serde_json::to_value(queue_items).unwrap()),
    }))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct ModerationQueueItemWithDetails {
    pub id: Uuid,
    pub content_type: String,
    pub content_id: Uuid,
    pub reported_by: Uuid,
    pub reason: Option<String>,
    pub priority: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub reported_by_name: Option<String>,
    pub video_title: Option<String>,
    pub comment_content: Option<String>,
    pub content_owner: Option<String>,
}

// Get moderation statistics
pub async fn get_moderation_stats(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    // Check if user has moderation permissions
    if !has_moderation_permissions(&user) {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let stats = get_moderation_statistics(pool).await;

    match stats {
        Ok(stats_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Moderation statistics retrieved".to_string(),
                data: Some(serde_json::to_value(stats_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get moderation stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get moderation statistics".to_string(),
                errors: None,
            }))
        }
    }
}

// Get moderation statistics
async fn get_moderation_statistics(
    pool: &DbPool,
) -> Result<ModerationStats, Box<dyn std::error::Error + Send + Sync>> {
    // Get total counts
    let totals = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE status = 'pending') as total_pending,
            COUNT(*) FILTER (WHERE status = 'reviewed') as total_reviewed,
            COUNT(*) FILTER (WHERE status LIKE 'approved%') as total_approved,
            COUNT(*) FILTER (WHERE status LIKE 'rejected%' OR status LIKE 'deleted%') as total_rejected
        FROM moderation_queue
        "#
    )
    .fetch_one(pool)
    .await?;

    // Get average review time
    let avg_review_time = sqlx::query!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (reviewed_at - created_at))) as avg_time
        FROM moderation_queue
        WHERE reviewed_at IS NOT NULL
        "#
    )
    .fetch_one(pool)
    .await?;

    // Get top violation types
    let violation_types = sqlx::query_as!(
        ViolationType,
        r#"
        SELECT 
            reason as violation_type,
            COUNT(*) as count,
            COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percentage
        FROM moderation_queue
        WHERE reason IS NOT NULL
        GROUP BY reason
        ORDER BY count DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    // Get moderator performance
    let moderator_performance = sqlx::query_as!(
        ModeratorPerformance,
        r#"
        SELECT 
            u.id as moderator_id,
            u.username as moderator_name,
            COUNT(mq.id) as items_reviewed,
            AVG(CASE WHEN ma.action IN ('approve', 'reject', 'delete') THEN 1.0 ELSE 0.0 END) as accuracy_rate,
            AVG(EXTRACT(EPOCH FROM (mq.reviewed_at - mq.created_at))) as avg_review_time,
            STRING_AGG(DISTINCT mq.reason, ', ') as top_violations_handled
        FROM moderation_queue mq
        JOIN users u ON mq.moderator_id = u.id
        LEFT JOIN moderation_actions ma ON mq.id = ma.content_id AND mq.content_type = ma.content_type
        WHERE mq.status = 'reviewed'
        GROUP BY u.id, u.username
        ORDER BY items_reviewed DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(ModerationStats {
        total_pending: totals.total_pending.unwrap_or(0),
        total_reviewed: totals.total_reviewed.unwrap_or(0),
        total_approved: totals.total_approved.unwrap_or(0),
        total_rejected: totals.total_rejected.unwrap_or(0),
        avg_review_time: avg_review_time.avg_time.unwrap_or(0.0),
        top_violation_types: violation_types,
        moderator_performance,
    })
}

// AI content analysis
pub async fn analyze_content_with_ai(
    pool: web::Data<DbPool>,
    content_id: web::Path<Uuid>,
    content_type: web::Path<String>,
) -> Result<HttpResponse> {
    let analysis = perform_ai_content_analysis(pool, content_id.into_inner(), content_type.into_inner()).await;

    match analysis {
        Ok(analysis_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "AI content analysis completed".to_string(),
                data: Some(serde_json::to_value(analysis_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("AI analysis failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to analyze content".to_string(),
                errors: None,
            }))
        }
    }
}

// Perform AI content analysis
async fn perform_ai_content_analysis(
    pool: &DbPool,
    content_id: Uuid,
    content_type: String,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    match content_type.as_str() {
        "video" => analyze_video_content_ai(pool, content_id).await,
        "comment" => analyze_comment_content_ai(pool, content_id).await,
        _ => Err("Unsupported content type".into()),
    }
}

// Analyze video content with AI
async fn analyze_video_content_ai(
    pool: &DbPool,
    video_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    // Get video information
    let video = sqlx::query!(
        "SELECT title, description, video_url FROM videos WHERE id = $1",
        video_id
    )
    .fetch_one(pool)
    .await?;

    // In production, call AI service to analyze video content
    // For now, return placeholder results
    Ok(serde_json::json!({
        "video_id": video_id,
        "content_type": "video",
        "analysis": {
            "toxicity_score": 0.0,
            "violence_score": 0.0,
            "explicit_content_score": 0.0,
            "hate_speech_score": 0.0,
            "spam_score": 0.0,
            "category": "entertainment",
            "confidence": 0.95,
            "risk_level": "low",
            "detected_objects": ["person", "outdoor"],
            "sentiment": "positive",
            "topics": ["lifestyle", "travel"],
            "ai_recommendation": "approve"
        },
        "metadata": {
            "title": video.title,
            "description": video.description,
            "analysis_timestamp": Utc::now().to_rfc3339()
        }
    }))
}

// Analyze comment content with AI
async fn analyze_comment_content_ai(
    pool: &DbPool,
    comment_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    // Get comment information
    let comment = sqlx::query!(
        "SELECT content, user_id FROM comments WHERE id = $1",
        comment_id
    )
    .fetch_one(pool)
    .await?;

    // In production, call AI service to analyze comment content
    // For now, return placeholder results
    Ok(serde_json::json!({
        "comment_id": comment_id,
        "content_type": "comment",
        "analysis": {
            "toxicity_score": 0.1,
            "violence_score": 0.0,
            "explicit_content_score": 0.0,
            "hate_speech_score": 0.0,
            "spam_score": 0.2,
            "category": "general",
            "confidence": 0.90,
            "risk_level": "low",
            "sentiment": "neutral",
            "language": "english",
            "ai_recommendation": "approve"
        },
        "metadata": {
            "content": comment.content,
            "user_id": comment.user_id,
            "analysis_timestamp": Utc::now().to_rfc3339()
        }
    }))
}

// Report content
pub async fn report_content(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    req: web::Json<ReportContentRequest>,
) -> Result<HttpResponse> {
    // Validate content exists
    let content_exists = verify_content_exists(pool, &req.content_type, req.content_id).await;

    if !content_exists {
        return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
            success: false,
            message: "Content not found".to_string(),
            errors: None,
        }));
    }

    // Add to moderation queue
    let queue_id = Uuid::new_v4();
    let now = Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO moderation_queue (id, content_type, content_id, reported_by, reason, priority, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        queue_id,
        req.content_type,
        req.content_id,
        user.id,
        req.reason,
        calculate_priority(&req.reason),
        "pending",
        now
    )
    .fetch_one(pool.as_ref())
    .await {
        Ok(_) => {
            // Record report
            if let Err(e) = record_user_report(pool, user.id, queue_id, &req.content_type, req.content_id, &req.reason).await {
                eprintln!("Failed to record user report: {}", e);
            }

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Content reported successfully".to_string(),
                data: Some(serde_json::json!({"report_id": queue_id})),
            }))
        }
        Err(e) => {
            eprintln!("Failed to add to moderation queue: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to report content".to_string(),
                errors: None,
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportContentRequest {
    pub content_type: String,
    pub content_id: Uuid,
    pub reason: String,
}

// Verify content exists
async fn verify_content_exists(
    pool: &DbPool,
    content_type: &str,
    content_id: Uuid,
) -> bool {
    match content_type {
        "video" => {
            let result = sqlx::query!(
                "SELECT id FROM videos WHERE id = $1",
                content_id
            )
            .fetch_optional(pool)
            .await;
            result.is_ok() && result.unwrap().is_some()
        }
        "comment" => {
            let result = sqlx::query!(
                "SELECT id FROM comments WHERE id = $1",
                content_id
            )
            .fetch_optional(pool)
            .await;
            result.is_ok() && result.unwrap().is_some()
        }
        "user" => {
            let result = sqlx::query!(
                "SELECT id FROM users WHERE id = $1",
                content_id
            )
            .fetch_optional(pool)
            .await;
            result.is_ok() && result.unwrap().is_some()
        }
        _ => false,
    }
}

// Calculate priority based on reason
fn calculate_priority(reason: &str) -> i32 {
    let priority_reasons = HashMap::from([
        ("violence".to_string(), 5),
        ("hate_speech".to_string(), 5),
        ("harassment".to_string(), 5),
        ("spam".to_string(), 3),
        ("inappropriate".to_string(), 2),
        ("copyright".to_string(), 4),
        ("other".to_string(), 1),
    ]);

    priority_reasons.get(&reason.to_lowercase()).unwrap_or(&1).clone()
}

// Record user report
async fn record_user_report(
    pool: &DbPool,
    user_id: Uuid,
    report_id: Uuid,
    content_type: &str,
    content_id: Uuid,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO user_reports (user_id, report_id, content_type, content_id, reason, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        report_id,
        content_type,
        content_id,
        reason,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Get user report history
pub async fn get_user_report_history(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    let reports = sqlx::query_as!(
        UserReport,
        r#"
        SELECT 
            ur.id, ur.content_type, ur.content_id, ur.reason, ur.created_at,
            mq.status as moderation_status,
            mq.reviewed_at
        FROM user_reports ur
        LEFT JOIN moderation_queue mq ON ur.report_id = mq.id
        WHERE ur.user_id = $1
        ORDER BY ur.created_at DESC
        LIMIT 50
        "#,
        user.id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Report history retrieved".to_string(),
        data: Some(serde_json::to_value(reports).unwrap()),
    }))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct UserReport {
    pub id: Uuid,
    pub content_type: String,
    pub content_id: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub moderation_status: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}