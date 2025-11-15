use actix_web::{HttpResponse};
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: false,
    pub message: String,
    pub errors: Option<ValidationErrors>,
}

// Create success response
pub fn success_response<T: Serialize>(message: &str, data: Option<T>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: message.to_string(),
        data: data.map(|d| serde_json::to_value(d).unwrap_or_default()),
    })
}

// Create error response
pub fn error_response(message: &str, errors: Option<ValidationErrors>) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
        success: false,
        message: message.to_string(),
        errors,
    })
}

// Create not found response
pub fn not_found_response(message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResponse {
        success: false,
        message: message.to_string(),
        errors: None,
    })
}

// Create unauthorized response
pub fn unauthorized_response(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse {
        success: false,
        message: message.to_string(),
        errors: None,
    })
}

// Create forbidden response
pub fn forbidden_response(message: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(ErrorResponse {
        success: false,
        message: message.to_string(),
        errors: None,
    })
}

// Create internal server error response
pub fn internal_error_response(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse {
        success: false,
        message: message.to_string(),
        errors: None,
    })
}

// Pagination helper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            sort_by: None,
            sort_order: Some("desc".to_string()),
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.unwrap_or(1) - 1) * self.limit.unwrap_or(20)
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).min(100) // Cap at 100
    }
}

// Pagination response
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let total_pages = (total as f64 / limit as f64).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            items,
            total,
            page,
            limit,
            total_pages,
            has_next,
            has_prev,
        }
    }
}

// File upload helpers
pub struct FileUploadOptions {
    pub max_size: u64, // in bytes
    pub allowed_types: Vec<String>,
    pub allowed_extensions: Vec<String>,
}

impl Default for FileUploadOptions {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            allowed_types: vec
!["video/mp4", "video/quicktime", "video/x-msvideo"],
            allowed_extensions: vec
!["mp4", "mov", "avi"],
        }
    }
}

impl FileUploadOptions {
    pub fn validate_file(&self, filename: &str, content_type: &str, size: u64) -> Result<(), String> {
        // Check file size
        if size > self.max_size {
            return Err("File too large".to_string());
        }

        // Check content type
        if !self.allowed_types.contains(&content_type.to_string()) {
            return Err("Invalid file type".to_string());
        }

        // Check file extension
        let extension = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();

        if !self.allowed_extensions.contains(&extension) {
            return Err("Invalid file extension".to_string());
        }

        Ok(())
    }
}

// Generate unique filename
pub fn generate_unique_filename(original: &str) -> String {
    let ext = original
        .split('.')
        .last()
        .unwrap_or("bin")
        .to_string();

    format!("{}.{}", uuid::Uuid::new_v4(), ext)
}

// Video processing helpers
pub struct VideoProcessingOptions {
    pub output_formats: Vec<String>,
    pub quality_profiles: Vec<QualityProfile>,
    pub max_duration: i32, // in seconds
    pub max_resolution: (i32, i32), // width, height
}

#[derive(Debug, Clone)]
pub struct QualityProfile {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub bitrate: i32,
    pub audio_bitrate: i32,
}

impl Default for VideoProcessingOptions {
    fn default() -> Self {
        Self {
            output_formats: vec
!["mp4".to_string()],
            quality_profiles: vec
![
                QualityProfile {
                    name: "1080p".to_string(),
                    width: 1920,
                    height: 1080,
                    bitrate: 5000,
                    audio_bitrate: 128,
                },
                QualityProfile {
                    name: "720p".to_string(),
                    width: 1280,
                    height: 720,
                    bitrate: 3000,
                    audio_bitrate: 96,
                },
                QualityProfile {
                    name: "480p".to_string(),
                    width: 854,
                    height: 480,
                    bitrate: 1500,
                    audio_bitrate: 64,
                },
                QualityProfile {
                    name: "360p".to_string(),
                    width: 640,
                    height: 360,
                    bitrate: 800,
                    audio_bitrate: 48,
                },
            ],
            max_duration: 3600, // 1 hour
            max_resolution: (3840, 2160), // 4K
        }
    }
}

impl VideoProcessingOptions {
    pub fn validate_video(&self, duration: i32, width: i32, height: i32) -> Result<(), String> {
        // Check duration
        if duration > self.max_duration {
            return Err("Video too long".to_string());
        }

        // Check resolution
        if width > self.max_resolution.0 || height > self.max_resolution.1 {
            return Err("Video resolution too high".to_string());
        }

        Ok(())
    }

    pub fn get_quality_profile(&self, name: &str) -> Option<&QualityProfile> {
        self.quality_profiles.iter().find(|p| p.name == name)
    }
}

// Thumbnail generation helpers
pub struct ThumbnailOptions {
    pub formats: Vec<String>,
    pub sizes: Vec<(i32, i32)>,
    pub quality: i32, // 1-100
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            formats: vec
!["jpg".to_string()],
            sizes: vec
![
                (1920, 1080), // 1080p
                (1280, 720),  // 720p
                (854, 480),   // 480p
                (640, 360),   // 360p
                (320, 180),   // 180p
            ],
            quality: 85,
        }
    }
}

// AI analysis helpers
pub struct AIAnalysisOptions {
    pub models: Vec<String>,
    pub confidence_threshold: f64,
    pub max_analysis_time: u64, // in seconds
}

impl Default for AIAnalysisOptions {
    fn default() -> Self {
        Self {
            models: vec
!["content_analysis".to_string(), "sentiment_analysis".to_string()],
            confidence_threshold: 0.7,
            max_analysis_time: 30,
        }
    }
}

// Rate limiting helpers
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let window_start = now - self.window;

        // Clean old requests
        self.requests.entry(key.to_string()).or_insert_with(Vec::new)
            .retain(|&timestamp| timestamp > window_start);

        // Check if limit exceeded
        let requests = self.requests.get_mut(key).unwrap();
        if requests.len() >= self.max_requests as usize {
            false
        } else {
            requests.push(now);
            true
        }
    }
}

// Cache helpers
use std::collections::HashMap;
use std::time::Duration;

pub struct Cache<T> {
    data: HashMap<String, (T, Instant)>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        let now = Instant::now();
        
        if let Some((value, timestamp)) = self.data.get(key) {
            if now.duration_since(*timestamp) < self.ttl {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn set(&mut self, key: String, value: T) {
        self.data.insert(key, (value, Instant::now()));
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

// Search helpers
pub fn build_search_query(
    base_query: &str,
    filters: &HashMap<String, String>,
    sort_by: &Option<String>,
    sort_order: &Option<String>,
) -> (String, Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>>) {
    let mut query = base_query.to_string();
    let mut params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![];

    // Add filters
    for (key, value) in filters {
        query.push_str(&format!(" AND {} = ${}", key, params.len() + 1));
        params.push(Box::new(value.clone()));
    }

    // Add sorting
    if let Some(sort) = sort_by {
        let order = sort_order.as_ref().map_or("DESC", |s| s.as_str());
        query.push_str(&format!(" ORDER BY {} {}", sort, order));
    }

    (query, params)
}

// Validation helpers
pub fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8 && 
    password.chars().any(|c| c.is_ascii_uppercase()) &&
    password.chars().any(|c| c.is_ascii_lowercase()) &&
    password.chars().any(|c| c.is_ascii_digit()) &&
    password.chars().any(|c| !c.is_ascii_alphanumeric())
}

pub fn validate_username(username: &str) -> bool {
    let username_regex = regex::Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
    username_regex.is_match(username)
}

// Security helpers
use sha2::{Sha256, Digest};

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_salt() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let salt: [u8; 16] = rng.gen();
    hex::encode(salt)
}

pub fn sanitize_html(input: &str) -> String {
    // Basic HTML sanitization - in production, use a proper library like ammonia
    let mut output = String::new();
    let mut in_tag = false;
    
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(c),
            _ => (),
        }
    }
    
    output
}

// Analytics helpers
pub struct AnalyticsMetrics {
    pub views: i64,
    pub likes: i64,
    pub comments: i64,
    pub shares: i64,
    pub engagement_rate: f64,
    pub retention_rate: f64,
    pub watch_time: i64,
}

impl AnalyticsMetrics {
    pub fn calculate_engagement_rate(&self) -> f64 {
        let total_engagement = self.likes + self.comments + self.shares;
        if self.views > 0 {
            (total_engagement as f64 / self.views as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn calculate_retention_rate(&self, duration: i32) -> f64 {
        // Placeholder calculation
        if duration > 0 {
            (self.watch_time as f64 / (duration as f64 * self.views as f64)) * 100.0
        } else {
            0.0
        }
    }
}

// Notification helpers
#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

pub enum NotificationType {
    Like,
    Comment,
    Follow,
    System,
    Promotion,
}

impl From<NotificationType> for String {
    fn from(nt: NotificationType) -> Self {
        match nt {
            NotificationType::Like => "like".to_string(),
            NotificationType::Comment => "comment".to_string(),
            NotificationType::Follow => "follow".to_string(),
            NotificationType::System => "system".to_string(),
            NotificationType::Promotion => "promotion".to_string(),
        }
    }
}

// Export helpers
pub struct ExportOptions {
    pub format: String,
    pub quality: String,
    pub include_metadata: bool,
    pub watermark: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: "mp4".to_string(),
            quality: "720p".to_string(),
            include_metadata: true,
            watermark: None,
        }
    }
}

// Health check helpers
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime: Duration,
    pub database: HealthComponent,
    pub redis: HealthComponent,
    pub storage: HealthComponent,
}

#[derive(Debug, Clone)]
pub struct HealthComponent {
    pub status: String,
    pub response_time: Duration,
    pub details: Option<serde_json::Value>,
}

impl HealthComponent {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            response_time: Duration::from_millis(0),
            details: None,
        }
    }

    pub fn unhealthy(message: &str, response_time: Duration) -> Self {
        Self {
            status: format!("unhealthy: {}", message),
            response_time,
            details: None,
        }
    }
}