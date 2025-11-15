use actix_web::{web, HttpResponse, Responder, Result};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::{auth, database, utils};
use crate::database::DbPool;
use crate::config::Config;
use sqlx::FromRow;
use futures::TryStreamExt;
use std::io::Write;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UploadVideoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub video_url: String,
    pub duration: i32,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<i32>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<i32>,
    pub emotion_index: Option<f64>,
    pub topic_category: Option<String>,
    pub engagement_prediction: Option<f64>,
    pub ranking_score: f64,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub save_count: i64,
    pub status: String,
    pub visibility: String,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub processing_completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Video {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub video_url: String,
    pub duration: i32,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<i32>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<i32>,
    pub emotion_index: Option<f64>,
    pub topic_category: Option<String>,
    pub engagement_prediction: Option<f64>,
    pub ranking_score: f64,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub save_count: i64,
    pub status: String,
    pub visibility: String,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub processing_completed_at: Option<DateTime<Utc>>,
}

// Video upload with chunked support
pub async fn upload_video(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
    mut payload: Multipart,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let mut video_url = None;
    let mut thumbnail_url = None;
    let mut file_size = 0;
    let mut file_name = None;

    while let Some(mut field) = payload.next().await.transpose().unwrap() {
        let content_disposition = field.content_disposition().unwrap();
        file_name = Some(content_disposition
            .get_filename()
            .unwrap_or("unknown")
            .to_string());

        // Validate file type
        let file_name = file_name.as_ref().unwrap();
        if !file_name.ends_with(".mp4") && !file_name.ends_with(".mov") && !file_name.ends_with(".avi") {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Only MP4, MOV, and AVI files are allowed".to_string(),
                errors: None,
            }));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("upload_{}", Uuid::new_v4()));
        let mut file = File::create(&temp_path).await.unwrap();

        // Stream file content
        let mut bytes_received = 0;
        while let Some(chunk) = field.next().await.transpose().unwrap() {
            bytes_received += chunk.len();
            file.write_all(&chunk).await.unwrap();
        }
        file_size = bytes_received as i64;

        // Upload to S3
        let s3_key = format!("videos/{}/{}", user.id, Uuid::new_v4());
        video_url = Some(format!("{}/{}", config.s3.public_url, s3_key));

        // In production, upload to S3 here
        // For now, use placeholder URL
        video_url = Some(format!("https://example.com/videos/{}.mp4", Uuid::new_v4()));

        // Generate thumbnail
        thumbnail_url = Some(format!("https://example.com/thumbnails/{}.jpg", Uuid::new_v4()));

        // Clean up temp file
        tokio::fs::remove_file(&temp_path).await.unwrap();
    }

    if let (Some(video_url), Some(thumbnail_url)) = (video_url, thumbnail_url) {
        // Create video record
        let video_id = Uuid::new_v4();
        let now = Utc::now();

        match sqlx::query!(
            r#"
            INSERT INTO videos (id, user_id, title, video_url, thumbnail_url, size, status, visibility, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, title, description, thumbnail_url, video_url, duration, size, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, emotion_index, topic_category, engagement_prediction, ranking_score, view_count, like_count, comment_count, share_count, save_count, status, visibility, tags, metadata, created_at, updated_at, published_at, processing_completed_at
            "#,
            video_id,
            user.id,
            "Untitled Video", // Default title, will be updated by AI
            video_url,
            thumbnail_url,
            file_size,
            "processing",
            "public",
            now,
            now
        )
        .fetch_one(pool.as_ref())
        .await
        {
            Ok(video_data) => {
                let video = VideoResponse {
                    id: video_data.id,
                    user_id: video_data.user_id,
                    title: video_data.title,
                    description: video_data.description,
                    thumbnail_url: video_data.thumbnail_url,
                    video_url: video_data.video_url,
                    duration: video_data.duration,
                    size: video_data.size,
                    width: video_data.width,
                    height: video_data.height,
                    fps: video_data.fps,
                    bitrate: video_data.bitrate,
                    codec: video_data.codec,
                    audio_codec: video_data.audio_codec,
                    audio_bitrate: video_data.audio_bitrate,
                    emotion_index: video_data.emotion_index,
                    topic_category: video_data.topic_category,
                    engagement_prediction: video_data.engagement_prediction,
                    ranking_score: video_data.ranking_score,
                    view_count: video_data.view_count,
                    like_count: video_data.like_count,
                    comment_count: video_data.comment_count,
                    share_count: video_data.share_count,
                    save_count: video_data.save_count,
                    status: video_data.status,
                    visibility: video_data.visibility,
                    tags: video_data.tags,
                    metadata: video_data.metadata,
                    created_at: video_data.created_at,
                    updated_at: video_data.updated_at,
                    published_at: video_data.published_at,
                    processing_completed_at: video_data.processing_completed_at,
                };

                // Start video processing in background
                tokio::spawn(process_video(
                    video_id,
                    user.id,
                    video_url.clone(),
                    thumbnail_url.clone(),
                    pool.clone(),
                    config.clone(),
                ));

                Ok(HttpResponse::Accepted().json(utils::ApiResponse {
                    success: true,
                    message: "Video uploaded successfully. Processing...".to_string(),
                    data: Some(serde_json::to_value(video).unwrap()),
                }))
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to create video record".to_string(),
                    errors: None,
                }))
            }
        }
    } else {
        Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "No file uploaded".to_string(),
            errors: None,
        }))
    }
}

// Background video processing
async fn process_video(
    video_id: Uuid,
    user_id: Uuid,
    video_url: String,
    thumbnail_url: String,
    pool: DbPool,
    config: Config,
) {
    // Update video status to processing
    if let Err(e) = sqlx::query!(
        "UPDATE videos SET status = 'processing' WHERE id = $1",
        video_id
    )
    .execute(&pool)
    .await
    {
        eprintln!("Failed to update video status: {}", e);
        return;
    }

    // Extract video metadata
    let metadata = extract_video_metadata(&video_url).await;
    
    // Generate AI-powered title, description, and tags
    let ai_content = generate_ai_content(&video_url, &metadata).await;
    
    // Analyze video content
    let analysis = analyze_video_content(&video_url).await;

    // Update video with processed data
    let now = Utc::now();
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE videos 
        SET title = $1, description = $2, tags = $3, duration = $4, width = $5, height = $6, 
            fps = $7, bitrate = $8, codec = $9, audio_codec = $10, audio_bitrate = $11,
            emotion_index = $12, topic_category = $13, engagement_prediction = $14,
            ranking_score = $15, status = 'ready', processing_completed_at = $16
        WHERE id = $17
        "#,
        ai_content.title,
        ai_content.description,
        serde_json::to_string(&ai_content.tags).ok(),
        metadata.duration,
        metadata.width,
        metadata.height,
        metadata.fps,
        metadata.bitrate,
        metadata.codec,
        metadata.audio_codec,
        metadata.audio_bitrate,
        analysis.emotion_index,
        analysis.topic_category,
        analysis.engagement_prediction,
        analysis.ranking_score,
        now,
        video_id
    )
    .execute(&pool)
    .await
    {
        eprintln!("Failed to update video metadata: {}", e);
    }

    // Add to search index
    if let Err(e) = add_to_search_index(&pool, video_id, &ai_content, &analysis).await {
        eprintln!("Failed to add to search index: {}", e);
    }

    // Update user analytics
    if let Err(e) = update_user_analytics(&pool, user_id).await {
        eprintln!("Failed to update user analytics: {}", e);
    }
}

// Video metadata extraction
struct VideoMetadata {
    duration: i32,
    width: Option<i32>,
    height: Option<i32>,
    fps: Option<i32>,
    bitrate: Option<i32>,
    codec: Option<String>,
    audio_codec: Option<String>,
    audio_bitrate: Option<i32>,
}

async fn extract_video_metadata(video_url: &str) -> VideoMetadata {
    // In production, use FFmpeg or similar tool to extract metadata
    // For now, return placeholder values
    VideoMetadata {
        duration: 60, // 1 minute placeholder
        width: Some(1920),
        height: Some(1080),
        fps: Some(30),
        bitrate: Some(5000),
        codec: Some("h264".to_string()),
        audio_codec: Some("aac".to_string()),
        audio_bitrate: Some(128),
    }
}

// AI content generation
struct AIContent {
    title: String,
    description: Option<String>,
    tags: Vec<String>,
}

async fn generate_ai_content(video_url: &str, metadata: &VideoMetadata) -> AIContent {
    // In production, call AI service to generate content
    // For now, return placeholder values
    AIContent {
        title: "AI-Generated Title".to_string(),
        description: Some("This is an AI-generated description for the video.".to_string()),
        tags: vec!["video".to_string(), "content".to_string()],
    }
}

// Video content analysis
struct VideoAnalysis {
    emotion_index: Option<f64>,
    topic_category: Option<String>,
    engagement_prediction: Option<f64>,
    ranking_score: f64,
}

async fn analyze_video_content(video_url: &str) -> VideoAnalysis {
    // In production, call AI service to analyze video content
    // For now, return placeholder values
    VideoAnalysis {
        emotion_index: Some(0.75),
        topic_category: Some("entertainment".to_string()),
        engagement_prediction: Some(0.8),
        ranking_score: 85.0,
    }
}

// Add to search index
async fn add_to_search_index(
    pool: &DbPool,
    video_id: Uuid,
    content: &AIContent,
    analysis: &VideoAnalysis,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In production, add to Elasticsearch
    // For now, skip this step
    Ok(())
}

// Update user analytics
async fn update_user_analytics(pool: &DbPool, user_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update user's video count, total views, etc.
    // For now, skip this step
    Ok(())
}

// Get video
pub async fn get_video(
    pool: web::Data<DbPool>,
    video_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let video = sqlx::query_as!(
        Video,
        r#"
        SELECT id, user_id, title, description, thumbnail_url, video_url, duration, size, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, emotion_index, topic_category, engagement_prediction, ranking_score, view_count, like_count, comment_count, share_count, save_count, status, visibility, tags, metadata, created_at, updated_at, published_at, processing_completed_at
        FROM videos
        WHERE id = $1 AND status = 'ready'
        "#,
        video_id.into_inner()
    )
    .fetch_optional(pool.as_ref())
    .await;

    match video {
        Ok(Some(video_data)) => {
            // Increment view count
            if let Err(e) = sqlx::query!(
                "UPDATE videos SET view_count = view_count + 1 WHERE id = $1",
                video_data.id
            )
            .execute(pool.as_ref())
            .await
            {
                eprintln!("Failed to increment view count: {}", e);
            }

            let video_response = VideoResponse {
                id: video_data.id,
                user_id: video_data.user_id,
                title: video_data.title,
                description: video_data.description,
                thumbnail_url: video_data.thumbnail_url,
                video_url: video_data.video_url,
                duration: video_data.duration,
                size: video_data.size,
                width: video_data.width,
                height: video_data.height,
                fps: video_data.fps,
                bitrate: video_data.bitrate,
                codec: video_data.codec,
                audio_codec: video_data.audio_codec,
                audio_bitrate: video_data.audio_bitrate,
                emotion_index: video_data.emotion_index,
                topic_category: video_data.topic_category,
                engagement_prediction: video_data.engagement_prediction,
                ranking_score: video_data.ranking_score,
                view_count: video_data.view_count,
                like_count: video_data.like_count,
                comment_count: video_data.comment_count,
                share_count: video_data.share_count,
                save_count: video_data.save_count,
                status: video_data.status,
                visibility: video_data.visibility,
                tags: video_data.tags,
                metadata: video_data.metadata,
                created_at: video_data.created_at,
                updated_at: video_data.updated_at,
                published_at: video_data.published_at,
                processing_completed_at: video_data.processing_completed_at,
            };

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Video retrieved".to_string(),
                data: Some(serde_json::to_value(video_response).unwrap()),
            }))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
            success: false,
            message: "Video not found".to_string(),
            errors: None,
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to retrieve video".to_string(),
                errors: None,
            }))
        }
    }
}

// Get video metadata
pub async fn get_video_metadata(
    pool: web::Data<DbPool>,
    video_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let video = sqlx::query_as!(
        Video,
        r#"
        SELECT id, user_id, title, description, thumbnail_url, video_url, duration, size, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, emotion_index, topic_category, engagement_prediction, ranking_score, view_count, like_count, comment_count, share_count, save_count, status, visibility, tags, metadata, created_at, updated_at, published_at, processing_completed_at
        FROM videos
        WHERE id = $1
        "#,
        video_id.into_inner()
    )
    .fetch_optional(pool.as_ref())
    .await;

    match video {
        Ok(Some(video_data)) => {
            let metadata_response = serde_json::json!({
                "id": video_data.id,
                "title": video_data.title,
                "description": video_data.description,
                "thumbnail_url": video_data.thumbnail_url,
                "duration": video_data.duration,
                "size": video_data.size,
                "width": video_data.width,
                "height": video_data.height,
                "fps": video_data.fps,
                "bitrate": video_data.bitrate,
                "codec": video_data.codec,
                "audio_codec": video_data.audio_codec,
                "audio_bitrate": video_data.audio_bitrate,
                "emotion_index": video_data.emotion_index,
                "topic_category": video_data.topic_category,
                "engagement_prediction": video_data.engagement_prediction,
                "ranking_score": video_data.ranking_score,
                "view_count": video_data.view_count,
                "like_count": video_data.like_count,
                "comment_count": video_data.comment_count,
                "share_count": video_data.share_count,
                "save_count": video_data.save_count,
                "status": video_data.status,
                "visibility": video_data.visibility,
                "tags": video_data.tags,
                "metadata": video_data.metadata,
                "created_at": video_data.created_at,
                "updated_at": video_data.updated_at,
                "published_at": video_data.published_at,
                "processing_completed_at": video_data.processing_completed_at,
            });

            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Video metadata retrieved".to_string(),
                data: Some(metadata_response),
            }))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
            success: false,
            message: "Video not found".to_string(),
            errors: None,
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to retrieve video metadata".to_string(),
                errors: None,
            }))
        }
    }
}

// Stream video
pub async fn stream_video(
    pool: web::Data<DbPool>,
    video_id: web::Path<Uuid>,
    range: Option<String>,
) -> Result<HttpResponse> {
    let video = sqlx::query!(
        "SELECT video_url, size FROM videos WHERE id = $1",
        video_id.into_inner()
    )
    .fetch_optional(pool.as_ref())
    .await;

    match video {
        Ok(Some(video_data)) => {
            // In production, stream video from S3 or CDN
            // For now, return placeholder response
            let content_type = "video/mp4";
            let content_length = video_data.size;

            if let Some(range_header) = range {
                // Handle range requests for partial content
                let ranges = parse_range_header(&range_header, content_length);
                
                if let Some((start, end)) = ranges {
                    let content_range = format!("bytes {}-{}/{}", start, end, content_length);
                    let content_length = (end - start + 1) as u64;
                    
                    Ok(HttpResponse::PartialContent()
                        .content_type(content_type)
                        .header("Content-Range", content_range)
                        .header("Accept-Ranges", "bytes")
                        .header("Content-Length", content_length)
                        .body(vec![0; content_length as usize])) // Placeholder
                } else {
                    Ok(HttpResponse::RangeNotSatisfiable().finish())
                }
            } else {
                Ok(HttpResponse::Ok()
                    .content_type(content_type)
                    .header("Content-Length", content_length)
                    .body(vec![0; content_length as usize])) // Placeholder
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
            success: false,
            message: "Video not found".to_string(),
            errors: None,
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to retrieve video".to_string(),
                errors: None,
            }))
        }
    }
}

// Parse range header
fn parse_range_header(range_header: &str, content_length: i64) -> Option<(i64, i64)> {
    // Parse range header format: "bytes=start-end"
    // For now, return placeholder values
    Some((0, content_length - 1))
}