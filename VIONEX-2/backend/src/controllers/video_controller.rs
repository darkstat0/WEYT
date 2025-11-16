use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::video::*;
use crate::repositories::video_repository::*;
use crate::services::video_service::*;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct VideoResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub user_id: Uuid,
    pub thumbnail_url: String,
    pub duration: i64,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub status: VideoStatus,
    pub visibility: VideoVisibility,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub stream_url: String,
}

#[derive(Debug, Serialize)]
pub struct VideoListResponse {
    pub videos: Vec<VideoResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateVideoRequest {
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub visibility: VideoVisibility,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVideoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<VideoVisibility>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchVideosQuery {
    pub query: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub duration_min: Option<i64>,
    pub duration_max: Option<i64>,
    pub quality_min: Option<VideoQuality>,
    pub upload_date: Option<String>,
    pub sort_by: Option<SortOption>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct VideoAnalyticsResponse {
    pub views_by_day: Vec<ViewByDay>,
    pub views_by_country: Vec<ViewByCountry>,
    pub views_by_device: Vec<ViewByDevice>,
    pub audience_retention: Vec<RetentionPoint>,
    pub engagement_metrics: EngagementMetrics,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize)]
pub struct VideoCommentResponse {
    pub id: Uuid,
    pub video_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub parent_id: Option<Uuid>,
    pub like_count: i64,
    pub is_reply: bool,
    pub is_pinned: bool,
    pub is_ai_generated: bool,
    pub toxicity_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub user: Option<UserSummary>,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Serialize)]
pub struct VideoPlaylistResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub video_count: i64,
    pub created_at: DateTime<Utc>,
    pub videos: Vec<PlaylistVideoSummary>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistVideoSummary {
    pub video_id: Uuid,
    pub title: String,
    pub thumbnail_url: String,
    pub duration: i64,
    pub position: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VideoRecommendationResponse {
    pub video_id: Uuid,
    pub score: f64,
    pub reason: String,
    pub confidence: f64,
    pub video: VideoSummary,
}

#[derive(Debug, Serialize)]
pub struct VideoSummary {
    pub id: Uuid,
    pub title: String,
    pub thumbnail_url: String,
    pub duration: i64,
    pub view_count: i64,
    pub like_count: i64,
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

pub async fn get_videos(
    query: web::Query<SearchVideosQuery>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = video_repo.search_videos(
        &query.query,
        query.category.as_ref(),
        &query.tags,
        query.duration_min,
        query.duration_max,
        query.quality_min.as_ref(),
        query.upload_date.as_ref(),
        query.sort_by.unwrap_or(SortOption::Relevance),
        page,
        page_size,
    ).await?;
    
    let response = VideoListResponse {
        videos: result.videos.into_iter().map(|video| VideoResponse {
            id: video.id,
            title: video.title,
            description: video.description,
            user_id: video.user_id,
            thumbnail_url: video.thumbnail_path,
            duration: video.duration,
            view_count: video.view_count,
            like_count: video.like_count,
            comment_count: video.comment_count,
            share_count: video.share_count,
            status: video.status,
            visibility: video.visibility,
            category: video.category,
            tags: video.tags,
            created_at: video.created_at,
            published_at: video.published_at,
            stream_url: format!("/api/v1/videos/{}/stream", video.id),
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_video(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let video = video_repo.get_video_by_id(&video_id).await?;
    
    let response = VideoResponse {
        id: video.id,
        title: video.title,
        description: video.description,
        user_id: video.user_id,
        thumbnail_url: video.thumbnail_path,
        duration: video.duration,
        view_count: video.view_count,
        like_count: video.like_count,
        comment_count: video.comment_count,
        share_count: video.share_count,
        status: video.status,
        visibility: video.visibility,
        category: video.category,
        tags: video.tags,
        created_at: video.created_at,
        published_at: video.published_at,
        stream_url: format!("/api/v1/videos/{}/stream", video.id),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn upload_video(
    req: web::Json<CreateVideoRequest>,
    video_service: web::Data<dyn VideoService>,
) -> AppResult<HttpResponse> {
    let video_data = req.into_inner();
    
    let video = video_service.create_video(
        &video_data.title,
        &video_data.description,
        video_data.category,
        video_data.tags,
        video_data.visibility,
        video_data.scheduled_at,
    ).await?;
    
    let response = VideoResponse {
        id: video.id,
        title: video.title,
        description: video.description,
        user_id: video.user_id,
        thumbnail_url: video.thumbnail_path,
        duration: video.duration,
        view_count: video.view_count,
        like_count: video.like_count,
        comment_count: video.comment_count,
        share_count: video.share_count,
        status: video.status,
        visibility: video.visibility,
        category: video.category,
        tags: video.tags,
        created_at: video.created_at,
        published_at: video.published_at,
        stream_url: format!("/api/v1/videos/{}/stream", video.id),
    };
    
    Ok(HttpResponse::Created().json(response))
}

pub async fn update_video(
    path: web::Path<Uuid>,
    req: web::Json<UpdateVideoRequest>,
    video_service: web::Data<dyn VideoService>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let update_data = req.into_inner();
    
    let video = video_service.update_video(
        &video_id,
        update_data.title,
        update_data.description,
        update_data.category,
        update_data.tags,
        update_data.visibility,
        update_data.scheduled_at,
    ).await?;
    
    let response = VideoResponse {
        id: video.id,
        title: video.title,
        description: video.description,
        user_id: video.user_id,
        thumbnail_url: video.thumbnail_path,
        duration: video.duration,
        view_count: video.view_count,
        like_count: video.like_count,
        comment_count: video.comment_count,
        share_count: video.share_count,
        status: video.status,
        visibility: video.visibility,
        category: video.category,
        tags: video.tags,
        created_at: video.created_at,
        published_at: video.published_at,
        stream_url: format!("/api/v1/videos/{}/stream", video.id),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_video(
    path: web::Path<Uuid>,
    video_service: web::Data<dyn VideoService>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    
    video_service.delete_video(&video_id).await?;
    
    Ok(HttpResponse::NoContent().finish())
}

pub async fn stream_video(
    path: web::Path<Uuid>,
    query: web::Query<VideoStreamRequest>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let stream_request = query.into_inner();
    
    let stream_info = video_repo.get_stream_info(&video_id, stream_request.quality, stream_request.format).await?;
    
    Ok(HttpResponse::Ok().json(stream_info))
}

pub async fn get_video_metadata(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let metadata = video_repo.get_video_metadata(&video_id).await?;
    
    Ok(HttpResponse::Ok().json(metadata))
}

pub async fn get_video_analytics(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let analytics = video_repo.get_video_analytics(&video_id).await?;
    
    let response = VideoAnalyticsResponse {
        views_by_day: analytics.views_by_day,
        views_by_country: analytics.views_by_country,
        views_by_device: analytics.views_by_device,
        audience_retention: analytics.audience_retention,
        engagement_metrics: analytics.engagement_metrics,
        performance_metrics: analytics.performance_metrics,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_video_comments(
    path: web::Path<Uuid>,
    query: web::Query<SearchVideosQuery>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = video_repo.get_video_comments(&video_id, &query.query, page, page_size).await?;
    
    let response = result.comments.into_iter().map(|comment| VideoCommentResponse {
        id: comment.id,
        video_id: comment.video_id,
        user_id: comment.user_id,
        content: comment.content,
        parent_id: comment.parent_id,
        like_count: comment.like_count,
        is_reply: comment.is_reply,
        is_pinned: comment.is_pinned,
        is_ai_generated: comment.is_ai_generated,
        toxicity_score: comment.toxicity_score,
        created_at: comment.created_at,
        user: result.users.iter().find(|u| u.id == comment.user_id).map(|u| UserSummary {
            id: u.id,
            username: u.username.clone(),
            avatar_url: u.avatar_url.clone(),
            is_verified: u.is_verified,
        }),
    }).collect();
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn add_video_comment(
    path: web::Path<Uuid>,
    req: web::Json<CreateCommentRequest>,
    video_service: web::Data<dyn VideoService>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let comment_data = req.into_inner();
    
    let comment = video_service.add_video_comment(&video_id, comment_data.content, comment_data.parent_id).await?;
    
    let response = VideoCommentResponse {
        id: comment.id,
        video_id: comment.video_id,
        user_id: comment.user_id,
        content: comment.content,
        parent_id: comment.parent_id,
        like_count: comment.like_count,
        is_reply: comment.is_reply,
        is_pinned: comment.is_pinned,
        is_ai_generated: comment.is_ai_generated,
        toxicity_score: comment.toxicity_score,
        created_at: comment.created_at,
        user: None, // Would be populated with user data
    };
    
    Ok(HttpResponse::Created().json(response))
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_id: Option<Uuid>,
}

pub async fn get_video_playlist(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let playlist_id = path.into_inner();
    let playlist = video_repo.get_video_playlist(&playlist_id).await?;
    
    let response = VideoPlaylistResponse {
        id: playlist.id,
        user_id: playlist.user_id,
        title: playlist.title,
        description: playlist.description,
        is_public: playlist.is_public,
        video_count: playlist.video_count,
        created_at: playlist.created_at,
        videos: playlist.videos.into_iter().map(|video| PlaylistVideoSummary {
            video_id: video.video_id,
            title: String::new(), // Would be populated from video data
            thumbnail_url: String::new(),
            duration: 0,
            position: video.position,
            added_at: video.added_at,
        }).collect(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_video_recommendations(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let recommendations = video_repo.get_video_recommendations(&video_id).await?;
    
    let response = recommendations.into_iter().map(|rec| VideoRecommendationResponse {
        video_id: rec.video_id,
        score: rec.score,
        reason: rec.reason,
        confidence: rec.confidence,
        video: VideoSummary {
            id: rec.video_id,
            title: String::new(),
            thumbnail_url: String::new(),
            duration: 0,
            view_count: 0,
            like_count: 0,
            user_id: Uuid::new_v4(),
            username: String::new(),
            avatar_url: None,
            published_at: None,
        },
    }).collect();
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_trending_videos(
    query: web::Query<SearchVideosQuery>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = video_repo.get_trending_videos(page, page_size).await?;
    
    let response = VideoListResponse {
        videos: result.videos.into_iter().map(|video| VideoResponse {
            id: video.id,
            title: video.title,
            description: video.description,
            user_id: video.user_id,
            thumbnail_url: video.thumbnail_path,
            duration: video.duration,
            view_count: video.view_count,
            like_count: video.like_count,
            comment_count: video.comment_count,
            share_count: video.share_count,
            status: video.status,
            visibility: video.visibility,
            category: video.category,
            tags: video.tags,
            created_at: video.created_at,
            published_at: video.published_at,
            stream_url: format!("/api/v1/videos/{}/stream", video.id),
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_related_videos(
    path: web::Path<Uuid>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let video_id = path.into_inner();
    let related_videos = video_repo.get_related_videos(&video_id).await?;
    
    let response = VideoListResponse {
        videos: related_videos.into_iter().map(|video| VideoResponse {
            id: video.id,
            title: video.title,
            description: video.description,
            user_id: video.user_id,
            thumbnail_url: video.thumbnail_path,
            duration: video.duration,
            view_count: video.view_count,
            like_count: video.like_count,
            comment_count: video.comment_count,
            share_count: video.share_count,
            status: video.status,
            visibility: video.visibility,
            category: video.category,
            tags: video.tags,
            created_at: video.created_at,
            published_at: video.published_at,
            stream_url: format!("/api/v1/videos/{}/stream", video.id),
        }).collect(),
        total: related_videos.len() as i64,
        page: 1,
        page_size: related_videos.len() as i32,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_popular_videos(
    query: web::Query<SearchVideosQuery>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = video_repo.get_popular_videos(page, page_size).await?;
    
    let response = VideoListResponse {
        videos: result.videos.into_iter().map(|video| VideoResponse {
            id: video.id,
            title: video.title,
            description: video.description,
            user_id: video.user_id,
            thumbnail_url: video.thumbnail_path,
            duration: video.duration,
            view_count: video.view_count,
            like_count: video.like_count,
            comment_count: video.comment_count,
            share_count: video.share_count,
            status: video.status,
            visibility: video.visibility,
            category: video.category,
            tags: video.tags,
            created_at: video.created_at,
            published_at: video.published_at,
            stream_url: format!("/api/v1/videos/{}/stream", video.id),
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_user_videos(
    path: web::Path<Uuid>,
    query: web::Query<SearchVideosQuery>,
    video_repo: web::Data<dyn VideoRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = video_repo.get_user_videos(&user_id, &query.query, page, page_size).await?;
    
    let response = VideoListResponse {
        videos: result.videos.into_iter().map(|video| VideoResponse {
            id: video.id,
            title: video.title,
            description: video.description,
            user_id: video.user_id,
            thumbnail_url: video.thumbnail_path,
            duration: video.duration,
            view_count: video.view_count,
            like_count: video.like_count,
            comment_count: video.comment_count,
            share_count: video.share_count,
            status: video.status,
            visibility: video.visibility,
            category: video.category,
            tags: video.tags,
            created_at: video.created_at,
            published_at: video.published_at,
            stream_url: format!("/api/v1/videos/{}/stream", video.id),
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}