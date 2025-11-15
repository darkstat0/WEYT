use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{auth, database, utils};
use crate::database::DbPool;
use crate::config::Config;
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub user_id: Option<Uuid>,
    pub video_id: Option<Uuid>,
    pub limit: Option<i32>,
    pub exclude_watched: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoRecommendation {
    pub video_id: Uuid,
    pub score: f64,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub video_id: Uuid,
    pub analysis_type: String, // "content", "engagement", "sentiment"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub video_id: Uuid,
    pub analysis_type: String,
    pub results: serde_json::Value,
    pub confidence: f64,
    pub processing_time: f64,
}

// Get personalized recommendations
pub async fn recommendations(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    req: web::Json<RecommendationRequest>,
) -> Result<HttpResponse> {
    let limit = req.limit.unwrap_or(10);
    let exclude_watched = req.exclude_watched.unwrap_or(true);

    let recommendations = if let Some(user_id) = req.user_id {
        get_user_recommendations(&pool, user_id, limit, exclude_watched).await
    } else {
        get_trending_recommendations(&pool, limit).await
    };

    match recommendations {
        Ok(recs) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Recommendations retrieved".to_string(),
                data: Some(serde_json::to_value(recs).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get recommendations: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get recommendations".to_string(),
                errors: None,
            }))
        }
    }
}

// Get user-specific recommendations
async fn get_user_recommendations(
    pool: &DbPool,
    user_id: Uuid,
    limit: i32,
    exclude_watched: bool,
) -> Result<Vec<VideoRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
    // Get user's watch history and preferences
    let watched_videos = if exclude_watched {
        sqlx::query!(
            "SELECT video_id FROM video_views WHERE user_id = $1",
            user_id
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.video_id)
        .collect::<Vec<Uuid>>()
    } else {
        Vec::new()
    };

    // Get user's liked videos
    let liked_videos = sqlx::query!(
        "SELECT video_id FROM likes WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| row.video_id)
    .collect::<Vec<Uuid>>();

    // Get user's subscriptions (if any)
    let subscriptions = sqlx::query!(
        "SELECT target_user_id FROM subscriptions WHERE subscriber_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| row.target_user_id)
    .collect::<Vec<Uuid>>();

    // Build recommendation query
    let mut query = String::from(
        r#"
        SELECT v.id, v.title, v.description, v.thumbnail_url, v.view_count, 
               v.like_count, v.comment_count, v.share_count, v.ranking_score,
               v.emotion_index, v.topic_category, v.engagement_prediction,
               u.username as creator_username
        FROM videos v
        JOIN users u ON v.user_id = u.id
        WHERE v.status = 'ready' AND v.visibility = 'public'
        "#,
    );

    let mut params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![];

    // Add watched videos exclusion
    if !watched_videos.is_empty() {
        let placeholders: Vec<String> = (1..=watched_videos.len())
            .map(|i| format!("${}", params.len() + i))
            .collect();
        query.push_str(&format!(" AND v.id NOT IN ({})", placeholders.join(",")));
        
        for video_id in watched_videos {
            params.push(Box::new(video_id));
        }
    }

    // Add subscription boost
    if !subscriptions.is_empty() {
        let placeholders: Vec<String> = (1..=subscriptions.len())
            .map(|i| format!("${}", params.len() + i))
            .collect();
        query.push_str(&format!(
            " OR v.user_id IN ({})",
            placeholders.join(",")
        ));
        
        for user_id in subscriptions {
            params.push(Box::new(user_id));
        }
    }

    // Add ordering
    query.push_str(
        r#"
        ORDER BY 
            v.ranking_score DESC,
            v.engagement_prediction DESC,
            v.view_count DESC,
            v.like_count DESC
        LIMIT $#
        "#,
    );

    params.push(Box::new(limit));

    // Execute query
    let videos = sqlx::query_as::<_, VideoWithCreator>(query.as_str())
        .bind_all(params)
        .fetch_all(pool)
        .await?;

    // Transform to recommendations with scores
    let mut recommendations = Vec::new();
    
    for video in videos {
        let mut score = video.ranking_score;
        
        // Boost subscription content
        if subscriptions.contains(&video.user_id) {
            score *= 1.5;
        }
        
        // Boost liked content types
        if liked_videos.contains(&video.id) {
            score *= 1.3;
        }

        // Calculate confidence based on engagement metrics
        let engagement = (video.view_count as f64 + video.like_count as f64 * 2.0 + 
                         video.comment_count as f64 * 3.0 + video.share_count as f64 * 4.0).sqrt();
        let confidence = (engagement / 10000.0).min(1.0);

        // Determine recommendation reason
        let reason = if subscriptions.contains(&video.user_id) {
            "Subscription".to_string()
        } else if liked_videos.contains(&video.id) {
            "Similar to liked videos".to_string()
        } else {
            "Trending".to_string()
        };

        recommendations.push(VideoRecommendation {
            video_id: video.id,
            score,
            reason,
            confidence,
        });
    }

    Ok(recommendations)
}

// Get trending recommendations
async fn get_trending_recommendations(
    pool: &DbPool,
    limit: i32,
) -> Result<Vec<VideoRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
    let videos = sqlx::query_as!(
        VideoWithCreator,
        r#"
        SELECT v.id, v.title, v.description, v.thumbnail_url, v.view_count, 
               v.like_count, v.comment_count, v.share_count, v.ranking_score,
               v.emotion_index, v.topic_category, v.engagement_prediction,
               u.username as creator_username
        FROM videos v
        JOIN users u ON v.user_id = u.id
        WHERE v.status = 'ready' AND v.visibility = 'public'
        ORDER BY v.ranking_score DESC, v.view_count DESC, v.like_count DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    let mut recommendations = Vec::new();
    
    for video in videos {
        let score = video.ranking_score * video.engagement_prediction.unwrap_or(0.0);
        
        recommendations.push(VideoRecommendation {
            video_id: video.id,
            score,
            reason: "Trending".to_string(),
            confidence: video.engagement_prediction.unwrap_or(0.0),
        });
    }

    Ok(recommendations)
}

// Analyze video content
pub async fn analyze_video(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    req: web::Json<AnalysisRequest>,
) -> Result<HttpResponse> {
    let start_time = std::time::Instant::now();

    // Get video data
    let video = sqlx::query!(
        "SELECT id, video_url, title, description, metadata FROM videos WHERE id = $1",
        req.video_id
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let video = match video {
        Some(video) => video,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Video not found".to_string(),
                errors: None,
            }));
        }
    };

    // Call AI service for analysis
    let analysis_results = match req.analysis_type.as_str() {
        "content" => analyze_video_content(&video.video_url).await,
        "engagement" => analyze_video_engagement(&video.id, pool).await,
        "sentiment" => analyze_video_sentiment(&video.title, &video.description).await,
        _ => {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Invalid analysis type".to_string(),
                errors: None,
            }));
        }
    };

    let processing_time = start_time.elapsed().as_secs_f64();

    let analysis_response = AnalysisResponse {
        video_id: video.id,
        analysis_type: req.analysis_type.clone(),
        results: serde_json::to_value(analysis_results).unwrap(),
        confidence: 0.85, // Placeholder confidence
        processing_time,
    };

    // Update video with analysis results
    if let Err(e) = update_video_analysis(pool, req.video_id, &analysis_results).await {
        eprintln!("Failed to update video analysis: {}", e);
    }

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Video analysis completed".to_string(),
        data: Some(serde_json::to_value(analysis_response).unwrap()),
    }))
}

// Analyze video content using AI
async fn analyze_video_content(video_url: &str) -> serde_json::Value {
    // In production, call AI service to analyze video content
    // For now, return placeholder results
    serde_json::json!({
        "objects": ["person", "car", "building"],
        "scenes": ["outdoor", "urban"],
        "actions": ["walking", "talking"],
        "objects_detected": 15,
        "scene_changes": 3,
        "dominant_colors": ["#FF6B6B", "#4ECDC4", "#45B7D1"],
        "faces_detected": 2,
        "text_detected": ["Hello World", "Welcome"]
    })
}

// Analyze video engagement
async fn analyze_video_engagement(video_id: &Uuid, pool: &DbPool) -> serde_json::Value {
    // Get engagement metrics
    let metrics = sqlx::query!(
        "SELECT view_count, like_count, comment_count, share_count, save_count 
         FROM videos WHERE id = $1",
        video_id
    )
    .fetch_one(pool)
    .await
    .unwrap();

    // Calculate engagement rate
    let total_engagement = metrics.like_count + metrics.comment_count + metrics.share_count + metrics.save_count;
    let engagement_rate = if metrics.view_count > 0 {
        (total_engagement as f64 / metrics.view_count as f64) * 100.0
    } else {
        0.0
    };

    // Analyze engagement patterns
    let peak_hours = analyze_peak_hours(video_id, pool).await;

    serde_json::json!({
        "view_count": metrics.view_count,
        "like_count": metrics.like_count,
        "comment_count": metrics.comment_count,
        "share_count": metrics.share_count,
        "save_count": metrics.save_count,
        "engagement_rate": engagement_rate,
        "total_engagement": total_engagement,
        "peak_hours": peak_hours,
        "retention_rate": 0.75, // Placeholder
        "completion_rate": 0.85, // Placeholder
        "engagement_growth": 0.15 // Placeholder
    })
}

// Analyze video sentiment
async fn analyze_video_sentiment(title: &str, description: &Option<String>) -> serde_json::Value {
    // In production, use NLP to analyze sentiment
    // For now, return placeholder results
    let text = format!("{} {}", title, description.as_deref().unwrap_or(""));
    
    serde_json::json!({
        "overall_sentiment": "positive",
        "sentiment_score": 0.75,
        "emotion": "happy",
        "keywords": ["awesome", "amazing", "great"],
        "topics": ["entertainment", "fun"],
        "language": "english",
        "toxicity_score": 0.0,
        "confidence": 0.9
    })
}

// Analyze peak viewing hours
async fn analyze_peak_hours(video_id: &Uuid, pool: &DbPool) -> Vec<i32> {
    // Get hourly view distribution
    let hourly_views = sqlx::query!(
        "SELECT EXTRACT(HOUR FROM created_at) as hour, COUNT(*) as views
         FROM video_views 
         WHERE video_id = $1 
         GROUP BY EXTRACT(HOUR FROM created_at)
         ORDER BY views DESC
         LIMIT 5",
        video_id
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    hourly_views
        .into_iter()
        .map(|row| row.hour as i32)
        .collect()
}

// Update video with analysis results
async fn update_video_analysis(
    pool: &DbPool,
    video_id: Uuid,
    analysis: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update video metadata with analysis results
    sqlx::query!(
        "UPDATE videos SET metadata = metadata || $1 WHERE id = $2",
        analysis,
        video_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Video similarity search
pub async fn find_similar_videos(
    pool: web::Data<DbPool>,
    video_id: web::Path<Uuid>,
    limit: Option<i32>,
) -> Result<HttpResponse> {
    let limit = limit.unwrap_or(5);

    // Get video details for similarity comparison
    let video = sqlx::query!(
        "SELECT title, description, topic_category, emotion_index, tags 
         FROM videos WHERE id = $1",
        video_id.into_inner()
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let video = match video {
        Some(video) => video,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Video not found".to_string(),
                errors: None,
            }));
        }
    };

    // Find similar videos based on various factors
    let similar_videos = find_similar_by_content(pool, &video, limit).await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Similar videos found".to_string(),
        data: Some(serde_json::to_value(similar_videos).unwrap()),
    }))
}

// Find similar videos by content
async fn find_similar_by_content(
    pool: &DbPool,
    video: &sqlx::postgres::PgRow,
    limit: i32,
) -> Result<Vec<VideoRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
    let topic_category = video.get::<Option<String>, _>("topic_category");
    let emotion_index = video.get::<Option<f64>, _>("emotion_index");
    let tags: Option<Vec<String>> = video.get("tags");

    let mut query = String::from(
        r#"
        SELECT v.id, v.title, v.description, v.thumbnail_url, v.view_count, 
               v.like_count, v.comment_count, v.share_count, v.ranking_score,
               v.emotion_index, v.topic_category, v.engagement_prediction,
               u.username as creator_username
        FROM videos v
        JOIN users u ON v.user_id = u.id
        WHERE v.id != $1 AND v.status = 'ready' AND v.visibility = 'public'
        "#,
    );

    let mut params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![Box::new(video.get::<Uuid, _>("id"))];

    // Add topic category filter
    if let Some(category) = topic_category {
        query.push_str(&format!(" AND v.topic_category = ${}", params.len() + 1));
        params.push(Box::new(category));
    }

    // Add emotion index similarity
    if let Some(emotion) = emotion_index {
        query.push_str(&format!(
            " AND ABS(v.emotion_index - ${}) < 0.3",
            params.len() + 1
        ));
        params.push(Box::new(emotion));
    }

    // Add tags similarity
    if let Some(video_tags) = tags {
        if !video_tags.is_empty() {
            let placeholders: Vec<String> = (1..=video_tags.len())
                .map(|i| format!("${}", params.len() + i))
                .collect();
            query.push_str(&format!(
                " AND v.tags && ARRAY[{}]",
                placeholders.join(",")
            ));
            
            for tag in video_tags {
                params.push(Box::new(tag));
            }
        }
    }

    // Add ordering by similarity
    query.push_str(
        r#"
        ORDER BY 
            (CASE WHEN v.topic_category = $2 THEN 1 ELSE 0 END) +
            (CASE WHEN ABS(v.emotion_index - $3) < 0.2 THEN 1 ELSE 0 END) +
            (CASE WHEN v.tags && $4 THEN 1 ELSE 0 END) DESC,
            v.ranking_score DESC
        LIMIT $5
        "#,
    );

    // Add additional parameters
    if let Some(category) = topic_category {
        params.push(Box::new(category));
    } else {
        params.push(Box::new(None::<String>));
    }

    if let Some(emotion) = emotion_index {
        params.push(Box::new(emotion));
    } else {
        params.push(Box::new(None::<f64>));
    }

    if let Some(video_tags) = tags {
        params.push(Box::new(video_tags));
    } else {
        params.push(Box::new(Vec::<String>::new()));
    }

    params.push(Box::new(limit));

    let videos = sqlx::query_as::<_, VideoWithCreator>(query.as_str())
        .bind_all(params)
        .fetch_all(pool)
        .await?;

    let mut recommendations = Vec::new();
    
    for video in videos {
        let score = video.ranking_score * video.engagement_prediction.unwrap_or(0.0);
        
        recommendations.push(VideoRecommendation {
            video_id: video.id,
            score,
            reason: "Similar content".to_string(),
            confidence: video.engagement_prediction.unwrap_or(0.0),
        });
    }

    Ok(recommendations)
}

// Helper struct for video with creator info
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct VideoWithCreator {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub ranking_score: f64,
    pub emotion_index: Option<f64>,
    pub topic_category: Option<String>,
    pub engagement_prediction: Option<f64>,
    pub creator_username: String,
}

// AI-powered video search
pub async fn ai_search(
    pool: web::Data<DbPool>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let search_query = query.get("q").and_then(|v| v.as_str()).unwrap_or("");
    let limit = query.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as i32;

    // Use AI to understand search intent
    let search_intent = understand_search_intent(search_query).await;

    // Perform semantic search
    let results = semantic_search(pool, search_query, &search_intent, limit).await?;

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Search results retrieved".to_string(),
        data: Some(serde_json::to_value(results).unwrap()),
    }))
}

// Understand search intent
async fn understand_search_intent(query: &str) -> serde_json::Value {
    // In production, use NLP to understand search intent
    // For now, return placeholder results
    serde_json::json!({
        "intent": "informational",
        "entities": ["video", "tutorial", "howto"],
        "sentiment": "neutral",
        "topic": "education",
        "confidence": 0.85
    })
}

// Semantic search using AI
async fn semantic_search(
    pool: &DbPool,
    query: &str,
    intent: &serde_json::Value,
    limit: i32,
) -> Result<Vec<VideoRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
    // Extract entities from intent
    let entities = intent.get("entities").and_then(|e| e.as_array()).unwrap_or(&vec
![]);

    // Build search query
    let mut search_query = String::from(
        r#"
        SELECT v.id, v.title, v.description, v.thumbnail_url, v.view_count, 
               v.like_count, v.comment_count, v.share_count, v.ranking_score,
               v.emotion_index, v.topic_category, v.engagement_prediction,
               u.username as creator_username,
               ts_rank_cd(v.search_vector, websearch_to_tsquery($1)) as rank
        FROM videos v
        JOIN users u ON v.user_id = u.id
        WHERE v.status = 'ready' AND v.visibility = 'public'
          AND v.search_vector @@ websearch_to_tsquery($1)
        "#,
    );

    // Add filters based on entities
    if !entities.is_empty() {
        let placeholders: Vec<String> = (2..=entities.len() + 1)
            .map(|i| format!("${}", i))
            .collect();
        search_query.push_str(&format!(
            " AND v.topic_category IN ({})",
            placeholders.join(",")
        ));
    }

    // Add ordering
    search_query.push_str(" ORDER BY rank DESC, v.ranking_score DESC LIMIT $");

    let total_params = entities.len() + 2;
    search_query.push_str(&total_params.to_string());

    // Execute search
    let mut query_builder = sqlx::query_as::<_, VideoWithCreator>(&search_query)
        .bind(query);

    for entity in entities {
        if let Some(entity_str) = entity.as_str() {
            query_builder = query_builder.bind(entity_str);
        }
    }

    query_builder = query_builder.bind(limit);

    let videos = query_builder.fetch_all(pool).await?;

    let mut recommendations = Vec::new();
    
    for video in videos {
        let score = video.ranking_score * video.engagement_prediction.unwrap_or(0.0);
        
        recommendations.push(VideoRecommendation {
            video_id: video.id,
            score,
            reason: "Search result".to_string(),
            confidence: video.engagement_prediction.unwrap_or(0.0),
        });
    }

    Ok(recommendations)
}