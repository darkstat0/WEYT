use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Video {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub user_id: Uuid,
    pub file_path: String,
    pub thumbnail_path: String,
    pub duration: i64,
    pub file_size: i64,
    pub format: String,
    pub resolution: VideoResolution,
    pub bitrate: i32,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub status: VideoStatus,
    pub visibility: VideoVisibility,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub ai_analysis: Option<VideoAIAnalysis>,
    pub metadata: VideoMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "video_resolution")]
pub struct VideoResolution {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "video_status")]
pub enum VideoStatus {
    Pending,
    Processing,
    Ready,
    Published,
    Private,
    Unlisted,
    Deleted,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "video_visibility")]
pub enum VideoVisibility {
    Public,
    Private,
    Unlisted,
    SubscribersOnly,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoMetadata {
    pub duration: i64,
    pub bitrate: i32,
    pub frame_rate: f32,
    pub codec: String,
    pub audio_codec: String,
    pub has_audio: bool,
    pub has_subtitles: bool,
    pub aspect_ratio: f32,
    pub color_space: String,
    pub file_hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoAIAnalysis {
    pub emotion_index: f64,
    pub topic_category: String,
    pub engagement_prediction: f64,
    pub ranking_score: f64,
    pub content_flags: Vec<ContentFlag>,
    pub auto_generated_tags: Vec<String>,
    pub auto_generated_description: Option<String>,
    pub auto_generated_title: Option<String>,
    pub thumbnail_quality_score: f64,
    pub audio_quality_score: f64,
    pub video_quality_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentFlag {
    pub flag_type: String,
    pub confidence: f64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoUpload {
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub visibility: VideoVisibility,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<VideoVisibility>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoStreamRequest {
    pub video_id: Uuid,
    pub quality: VideoQuality,
    pub format: StreamFormat,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub subtitles_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VideoQuality {
    Auto,
    Low144p,
    Medium240p,
    Medium360p,
    High480p,
    High720p,
    High1080p,
    Ultra4K,
    Ultra8K,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StreamFormat {
    HLS,
    DASH,
    WebM,
    MP4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnalytics {
    pub views_by_day: Vec<ViewByDay>,
    pub views_by_country: Vec<ViewByCountry>,
    pub views_by_device: Vec<ViewByDevice>,
    pub audience_retention: Vec<RetentionPoint>,
    pub engagement_metrics: EngagementMetrics,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewByDay {
    pub date: String,
    pub views: i64,
    pub watch_time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewByCountry {
    pub country: String,
    pub views: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewByDevice {
    pub device_type: String,
    pub views: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetentionPoint {
    pub timestamp: f64,
    pub percentage: f64,
    pub viewers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub likes: i64,
    pub comments: i64,
    pub shares: i64,
    pub saves: i64,
    pub click_through_rate: f64,
    pub average_view_duration: f64,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub load_time: f64,
    pub buffering_events: i64,
    pub average_bitrate: f64,
    pub quality_switches: i64,
    pub dropped_frames: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoSearchQuery {
    pub query: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub duration_min: Option<i64>,
    pub duration_max: Option<i64>,
    pub quality_min: Option<VideoQuality>,
    pub upload_date: Option<String>,
    pub sort_by: SortOption,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortOption {
    Relevance,
    Newest,
    Oldest,
    MostViewed,
    MostLiked,
    MostCommented,
    Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoSearchResult {
    pub videos: Vec<Video>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoComment {
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
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoPlaylist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub video_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub videos: Vec<PlaylistVideo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistVideo {
    pub video_id: Uuid,
    pub position: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoRecommendation {
    pub video_id: Uuid,
    pub score: f64,
    pub reason: String,
    pub confidence: f64,
}

impl Video {
    pub fn new(
        title: String,
        description: String,
        user_id: Uuid,
        file_path: String,
        thumbnail_path: String,
        duration: i64,
        file_size: i64,
        format: String,
        resolution: VideoResolution,
        bitrate: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            user_id,
            file_path,
            thumbnail_path,
            duration,
            file_size,
            format,
            resolution,
            bitrate,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            share_count: 0,
            status: VideoStatus::Pending,
            visibility: VideoVisibility::Private,
            category: None,
            tags: Vec::new(),
            ai_analysis: None,
            metadata: VideoMetadata {
                duration,
                bitrate,
                frame_rate: 30.0,
                codec: format.clone(),
                audio_codec: "aac".to_string(),
                has_audio: true,
                has_subtitles: false,
                aspect_ratio: 16.0 / 9.0,
                color_space: "yuv420p".to_string(),
                file_hash: String::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            scheduled_at: None,
        }
    }

    pub fn is_published(&self) -> bool {
        matches!(self.status, VideoStatus::Published)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, VideoStatus::Ready)
    }

    pub fn can_view(&self, user_id: Option<Uuid>) -> bool {
        match self.visibility {
            VideoVisibility::Public => true,
            VideoVisibility::Unlisted => true,
            VideoVisibility::SubscribersOnly => {
                if let Some(user_id) = user_id {
                    // Check if user is subscribed to the channel
                    // This would be implemented in the service layer
                    true
                } else {
                    false
                }
            },
            VideoVisibility::Private => {
                if let Some(user_id) = user_id {
                    user_id == self.user_id
                } else {
                    false
                }
            },
        }
    }

    pub fn get_view_duration(&self) -> String {
        let minutes = self.duration / 60;
        let seconds = self.duration % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    pub fn get_file_size_formatted(&self) -> String {
        const KB: i64 = 1024;
        const MB: i64 = KB * 1024;
        const GB: i64 = MB * 1024;
        
        if self.file_size >= GB {
            format!("{:.2} GB", self.file_size as f64 / GB as f64)
        } else if self.file_size >= MB {
            format!("{:.2} MB", self.file_size as f64 / MB as f64)
        } else if self.file_size >= KB {
            format!("{:.2} KB", self.file_size as f64 / KB as f64)
        } else {
            format!("{} B", self.file_size)
        }
    }
}

impl VideoResolution {
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn is_wide_screen(&self) -> bool {
        self.get_aspect_ratio() > 1.5
    }

    pub fn is_portrait(&self) -> bool {
        self.get_aspect_ratio() < 0.8
    }

    pub fn is_square(&self) -> bool {
        (self.get_aspect_ratio() - 1.0).abs() < 0.1
    }
}

impl VideoStatus {
    pub fn can_edit(&self) -> bool {
        matches!(self, VideoStatus::Pending | VideoStatus::Processing | VideoStatus::Ready)
    }

    pub fn can_publish(&self) -> bool {
        matches!(self, VideoStatus::Ready)
    }

    pub fn can_delete(&self) -> bool {
        !matches!(self, VideoStatus::Published)
    }
}

impl VideoVisibility {
    pub fn is_public(&self) -> bool {
        matches!(self, VideoVisibility::Public)
    }

    pub fn is_private(&self) -> bool {
        matches!(self, VideoVisibility::Private)
    }

    pub fn is_unlisted(&self) -> bool {
        matches!(self, VideoVisibility::Unlisted)
    }
}