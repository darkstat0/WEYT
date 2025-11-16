use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AIModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: AIModelType,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub status: ModelStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "ai_model_type")]
pub enum AIModelType {
    Recommendation,
    ContentModeration,
    ContentGeneration,
    SpeechRecognition,
    ImageRecognition,
    NaturalLanguageProcessing,
    VideoAnalysis,
    EmotionDetection,
    TextToSpeech,
    SpeechToText,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "model_status")]
pub enum ModelStatus {
    Training,
    Ready,
    Deployed,
    Deprecated,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RecommendationEngine {
    pub id: Uuid,
    pub user_id: Uuid,
    pub video_id: Uuid,
    pub score: f64,
    pub reason: String,
    pub confidence: f64,
    pub factors: Vec<RecommendationFactor>,
    pub context: RecommendationContext,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationFactor {
    pub factor_type: String,
    pub weight: f64,
    pub value: f64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationContext {
    pub time_of_day: String,
    pub user_mood: Option<String>,
    pub device_type: String,
    pub location: Option<String>,
    pub session_duration: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ContentModeration {
    pub id: Uuid,
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub moderation_result: ModerationResult,
    pub ai_confidence: f64,
    pub human_review: Option<HumanReview>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "content_type")]
pub enum ContentType {
    Video,
    Comment,
    Title,
    Description,
    Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationResult {
    pub is_violating: bool,
    pub violation_types: Vec<ViolationType>,
    pub confidence_scores: Vec<ViolationScore>,
    pub suggested_action: ModerationAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViolationType {
    pub type_name: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViolationScore {
    pub violation_type: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "violation_severity")]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "moderation_action")]
pub enum ModerationAction {
    Allow,
    Review,
    Flag,
    Remove,
    BlockUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HumanReview {
    pub reviewed_by: Uuid,
    pub reviewed_at: DateTime<Utc>,
    pub decision: ModerationAction,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ContentGeneration {
    pub id: Uuid,
    pub user_id: Uuid,
    pub prompt: String,
    pub generated_content: GeneratedContent,
    pub model_used: String,
    pub generation_time: f64,
    pub cost: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedContent {
    pub content_type: GeneratedContentType,
    pub content: serde_json::Value,
    pub metadata: ContentMetadata,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "generated_content_type")]
pub enum GeneratedContentType {
    Title,
    Description,
    Thumbnail,
    Tags,
    VideoClip,
    Audio,
    Image,
    Caption,
    Summary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub duration: Option<i64>,
    pub resolution: Option<String>,
    pub format: Option<String>,
    pub size: Option<i64>,
    pub quality_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SpeechRecognition {
    pub id: Uuid,
    pub audio_file_path: String,
    pub transcript: String,
    pub language: String,
    pub confidence: f64,
    pub word_timestamps: Vec<WordTimestamp>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordTimestamp {
    pub word: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ImageRecognition {
    pub id: Uuid,
    pub image_path: String,
    pub objects: Vec<RecognizedObject>,
    pub faces: Vec<RecognizedFace>,
    pub scene: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecognizedObject {
    pub label: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecognizedFace {
    pub id: Uuid,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
    pub emotions: Vec<EmotionScore>,
    pub age: Option<i32>,
    pub gender: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmotionScore {
    pub emotion: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NaturalLanguageProcessing {
    pub id: Uuid,
    pub text: String,
    pub analysis: NLPAnalysis,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NLPAnalysis {
    pub sentiment: SentimentAnalysis,
    pub entities: Vec<Entity>,
    pub keywords: Vec<Keyword>,
    pub topics: Vec<Topic>,
    pub summary: Option<String>,
    pub language: String,
    pub toxicity_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub overall: SentimentScore,
    pub by_aspect: Vec<AspectSentiment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentScore {
    pub score: f64,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AspectSentiment {
    pub aspect: String,
    pub sentiment: SentimentScore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub type_name: String,
    pub confidence: f64,
    pub start_pos: i32,
    pub end_pos: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keyword {
    pub text: String,
    pub relevance: f64,
    pub frequency: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoAnalysis {
    pub id: Uuid,
    pub video_id: Uuid,
    pub analysis: VideoAnalysisResult,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnalysisResult {
    pub scene_changes: Vec<SceneChange>,
    pub objects: Vec<RecognizedObject>,
    pub faces: Vec<RecognizedFace>,
    pub text: Vec<TextRegion>,
    pub audio_analysis: AudioAnalysis,
    pub visual_quality: QualityMetrics,
    pub content_categories: Vec<CategoryScore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneChange {
    pub timestamp: f64,
    pub confidence: f64,
    pub scene_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextRegion {
    pub text: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
    pub timestamp: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub speech_segments: Vec<SpeechSegment>,
    pub music_segments: Vec<MusicSegment>,
    pub silence_segments: Vec<SilenceSegment>,
    pub volume_levels: Vec<VolumeLevel>,
    pub background_noise: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub speaker: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub genre: Option<String>,
    pub energy: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SilenceSegment {
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeLevel {
    pub timestamp: f64,
    pub level: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub sharpness: f64,
    pub brightness: f64,
    pub contrast: f64,
    pub color_balance: f64,
    pub stability: f64,
    pub overall_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: String,
    pub confidence: f64,
    pub subcategories: Vec<SubcategoryScore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubcategoryScore {
    pub name: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EmotionDetection {
    pub id: Uuid,
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub emotions: Vec<EmotionScore>,
    pub dominant_emotion: Option<String>,
    pub overall_emotion_score: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TextToSpeech {
    pub id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub voice_id: String,
    pub audio_path: String,
    pub duration: i64,
    pub language: String,
    pub speed: f64,
    pub pitch: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SpeechToText {
    pub id: Uuid,
    pub user_id: Uuid,
    pub audio_path: String,
    pub transcript: String,
    pub language: String,
    pub confidence: f64,
    pub word_timestamps: Vec<WordTimestamp>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub inference_time: f64,
    pub throughput: f64,
    pub memory_usage: f64,
    pub cpu_usage: f64,
}

impl AIModel {
    pub fn new(
        name: String,
        version: String,
        model_type: AIModelType,
        description: String,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            model_type,
            description,
            input_schema,
            output_schema,
            performance_metrics: None,
            status: ModelStatus::Training,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, ModelStatus::Ready | ModelStatus::Deployed)
    }

    pub fn is_deprecated(&self) -> bool {
        matches!(self.status, ModelStatus::Deprecated)
    }
}

impl ModelStatus {
    pub fn can_deploy(&self) -> bool {
        matches!(self, ModelStatus::Ready)
    }

    pub fn can_train(&self) -> bool {
        matches!(self, ModelStatus::Training | ModelStatus::Failed)
    }
}

impl ModerationAction {
    pub fn is_blocking(&self) -> bool {
        matches!(self, ModerationAction::Remove | ModerationAction::BlockUser)
    }

    pub fn needs_review(&self) -> bool {
        matches!(self, ModerationAction::Review | ModerationAction::Flag)
    }
}

impl GeneratedContentType {
    pub fn is_media(&self) -> bool {
        matches!(
            self,
            GeneratedContentType::VideoClip
                | GeneratedContentType::Audio
                | GeneratedContentType::Image
                | GeneratedContentType::Thumbnail
        )
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self,
            GeneratedContentType::Title
                | GeneratedContentType::Description
                | GeneratedContentType::Tags
                | GeneratedContentType::Caption
                | GeneratedContentType::Summary
        )
    }
}