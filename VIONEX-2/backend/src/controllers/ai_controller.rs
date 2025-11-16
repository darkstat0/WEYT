use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::ai::*;
use crate::repositories::ai_repository::*;
use crate::services::ai_service::*;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct AIModelResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: AIModelType,
    pub description: String,
    pub status: ModelStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AIModelListResponse {
    pub models: Vec<AIModelResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateAIModelRequest {
    pub name: String,
    pub version: String,
    pub model_type: AIModelType,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAIModelRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub status: Option<ModelStatus>,
}

#[derive(Debug, Deserialize)]
pub struct SearchAIModelsQuery {
    pub query: String,
    pub model_type: Option<AIModelType>,
    pub status: Option<ModelStatus>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub video_id: Uuid,
    pub score: f64,
    pub reason: String,
    pub confidence: f64,
    pub factors: Vec<RecommendationFactor>,
    pub context: RecommendationContext,
}

#[derive(Debug, Serialize)]
pub struct RecommendationListResponse {
    pub recommendations: Vec<RecommendationResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetRecommendationsRequest {
    pub user_id: Uuid,
    pub limit: Option<i32>,
    pub exclude_watched: Option<bool>,
    pub context: Option<RecommendationContext>,
}

#[derive(Debug, Serialize)]
pub struct ContentModerationResponse {
    pub id: Uuid,
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub moderation_result: ModerationResult,
    pub ai_confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ModerateContentRequest {
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub content: String,
    pub auto_moderate: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ContentGenerationResponse {
    pub id: Uuid,
    pub prompt: String,
    pub generated_content: GeneratedContent,
    pub model_used: String,
    pub generation_time: f64,
    pub cost: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateContentRequest {
    pub prompt: String,
    pub content_type: GeneratedContentType,
    pub model: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SpeechRecognitionResponse {
    pub id: Uuid,
    pub transcript: String,
    pub language: String,
    pub confidence: f64,
    pub word_timestamps: Vec<WordTimestamp>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TranscribeAudioRequest {
    pub audio_file_path: String,
    pub language: Option<String>,
    pub include_timestamps: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ImageRecognitionResponse {
    pub id: Uuid,
    pub objects: Vec<RecognizedObject>,
    pub faces: Vec<RecognizedFace>,
    pub scene: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeImageRequest {
    pub image_path: String,
    pub detect_objects: Option<bool>,
    pub detect_faces: Option<bool>,
    pub detect_scene: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NaturalLanguageProcessingResponse {
    pub id: Uuid,
    pub text: String,
    pub analysis: NLPAnalysis,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeTextRequest {
    pub text: String,
    pub sentiment: Option<bool>,
    pub entities: Option<bool>,
    pub keywords: Option<bool>,
    pub topics: Option<bool>,
    pub summary: Option<bool>,
    pub toxicity: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VideoAnalysisResponse {
    pub id: Uuid,
    pub video_id: Uuid,
    pub analysis: VideoAnalysisResult,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeVideoRequest {
    pub video_id: Uuid,
    pub scene_changes: Option<bool>,
    pub objects: Option<bool>,
    pub faces: Option<bool>,
    pub text: Option<bool>,
    pub audio: Option<bool>,
    pub quality: Option<bool>,
    pub categories: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct EmotionDetectionResponse {
    pub id: Uuid,
    pub content_id: Uuid,
    pub emotions: Vec<EmotionScore>,
    pub dominant_emotion: Option<String>,
    pub overall_emotion_score: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DetectEmotionsRequest {
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TextToSpeechResponse {
    pub id: Uuid,
    pub audio_path: String,
    pub duration: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSpeechRequest {
    pub text: String,
    pub voice_id: Option<String>,
    pub language: Option<String>,
    pub speed: Option<f64>,
    pub pitch: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SpeechToTextResponse {
    pub id: Uuid,
    pub transcript: String,
    pub language: String,
    pub confidence: f64,
    pub word_timestamps: Vec<WordTimestamp>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_ai_models(
    query: web::Query<SearchAIModelsQuery>,
    ai_repo: web::Data<dyn AIRepository>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = ai_repo.search_ai_models(
        &query.query,
        query.model_type.as_ref(),
        query.status.as_ref(),
        page,
        page_size,
    ).await?;
    
    let response = AIModelListResponse {
        models: result.models.into_iter().map(|model| AIModelResponse {
            id: model.id,
            name: model.name,
            version: model.version,
            model_type: model.model_type,
            description: model.description,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_ai_model(
    path: web::Path<Uuid>,
    ai_repo: web::Data<dyn AIRepository>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    let model = ai_repo.get_ai_model_by_id(&model_id).await?;
    
    let response = AIModelResponse {
        id: model.id,
        name: model.name,
        version: model.version,
        model_type: model.model_type,
        description: model.description,
        status: model.status,
        created_at: model.created_at,
        updated_at: model.updated_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_ai_model(
    req: web::Json<CreateAIModelRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let model_data = req.into_inner();
    
    let model = ai_service.create_ai_model(
        &model_data.name,
        &model_data.version,
        model_data.model_type,
        &model_data.description,
        model_data.input_schema,
        model_data.output_schema,
    ).await?;
    
    let response = AIModelResponse {
        id: model.id,
        name: model.name,
        version: model.version,
        model_type: model.model_type,
        description: model.description,
        status: model.status,
        created_at: model.created_at,
        updated_at: model.updated_at,
    };
    
    Ok(HttpResponse::Created().json(response))
}

pub async fn update_ai_model(
    path: web::Path<Uuid>,
    req: web::Json<UpdateAIModelRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    let update_data = req.into_inner();
    
    let model = ai_service.update_ai_model(
        &model_id,
        update_data.name,
        update_data.version,
        update_data.description,
        update_data.status,
    ).await?;
    
    let response = AIModelResponse {
        id: model.id,
        name: model.name,
        version: model.version,
        model_type: model.model_type,
        description: model.description,
        status: model.status,
        created_at: model.created_at,
        updated_at: model.updated_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_ai_model(
    path: web::Path<Uuid>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    
    ai_service.delete_ai_model(&model_id).await?;
    
    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_recommendations(
    req: web::Json<GetRecommendationsRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let recommendations = ai_service.get_recommendations(
        &request_data.user_id,
        request_data.limit,
        request_data.exclude_watched,
        request_data.context,
    ).await?;
    
    let response = RecommendationListResponse {
        recommendations: recommendations.into_iter().map(|rec| RecommendationResponse {
            video_id: rec.video_id,
            score: rec.score,
            reason: rec.reason,
            confidence: rec.confidence,
            factors: rec.factors,
            context: rec.context,
        }).collect(),
        total: recommendations.len() as i64,
        page: 1,
        page_size: recommendations.len() as i32,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn moderate_content(
    req: web::Json<ModerateContentRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let moderation = ai_service.moderate_content(
        &request_data.content_id,
        request_data.content_type,
        &request_data.content,
        request_data.auto_moderate,
    ).await?;
    
    let response = ContentModerationResponse {
        id: moderation.id,
        content_id: moderation.content_id,
        content_type: moderation.content_type,
        moderation_result: moderation.moderation_result,
        ai_confidence: moderation.ai_confidence,
        created_at: moderation.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn generate_content(
    req: web::Json<GenerateContentRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let generation = ai_service.generate_content(
        &request_data.prompt,
        request_data.content_type,
        request_data.model,
        request_data.parameters,
    ).await?;
    
    let response = ContentGenerationResponse {
        id: generation.id,
        prompt: generation.prompt,
        generated_content: generation.generated_content,
        model_used: generation.model_used,
        generation_time: generation.generation_time,
        cost: generation.cost,
        created_at: generation.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn transcribe_audio(
    req: web::Json<TranscribeAudioRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let transcription = ai_service.transcribe_audio(
        &request_data.audio_file_path,
        request_data.language,
        request_data.include_timestamps,
    ).await?;
    
    let response = SpeechRecognitionResponse {
        id: transcription.id,
        transcript: transcription.transcript,
        language: transcription.language,
        confidence: transcription.confidence,
        word_timestamps: transcription.word_timestamps,
        created_at: transcription.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn analyze_image(
    req: web::Json<AnalyzeImageRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let analysis = ai_service.analyze_image(
        &request_data.image_path,
        request_data.detect_objects,
        request_data.detect_faces,
        request_data.detect_scene,
    ).await?;
    
    let response = ImageRecognitionResponse {
        id: analysis.id,
        objects: analysis.objects,
        faces: analysis.faces,
        scene: analysis.scene,
        created_at: analysis.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn analyze_text(
    req: web::Json<AnalyzeTextRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let analysis = ai_service.analyze_text(
        &request_data.text,
        request_data.sentiment,
        request_data.entities,
        request_data.keywords,
        request_data.topics,
        request_data.summary,
        request_data.toxicity,
    ).await?;
    
    let response = NaturalLanguageProcessingResponse {
        id: analysis.id,
        text: analysis.text,
        analysis: analysis.analysis,
        created_at: analysis.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn analyze_video(
    req: web::Json<AnalyzeVideoRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let analysis = ai_service.analyze_video(
        &request_data.video_id,
        request_data.scene_changes,
        request_data.objects,
        request_data.faces,
        request_data.text,
        request_data.audio,
        request_data.quality,
        request_data.categories,
    ).await?;
    
    let response = VideoAnalysisResponse {
        id: analysis.id,
        video_id: analysis.video_id,
        analysis: analysis.analysis,
        created_at: analysis.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn detect_emotions(
    req: web::Json<DetectEmotionsRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let emotion_detection = ai_service.detect_emotions(
        &request_data.content_id,
        request_data.content_type,
        request_data.method,
    ).await?;
    
    let response = EmotionDetectionResponse {
        id: emotion_detection.id,
        content_id: emotion_detection.content_id,
        emotions: emotion_detection.emotions,
        dominant_emotion: emotion_detection.dominant_emotion,
        overall_emotion_score: emotion_detection.overall_emotion_score,
        created_at: emotion_detection.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn generate_speech(
    req: web::Json<GenerateSpeechRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let speech = ai_service.generate_speech(
        &request_data.text,
        request_data.voice_id,
        request_data.language,
        request_data.speed,
        request_data.pitch,
    ).await?;
    
    let response = TextToSpeechResponse {
        id: speech.id,
        audio_path: speech.audio_path,
        duration: speech.duration,
        created_at: speech.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_ai_model_performance(
    path: web::Path<Uuid>,
    ai_repo: web::Data<dyn AIRepository>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    let performance = ai_repo.get_ai_model_performance(&model_id).await?;
    
    Ok(HttpResponse::Ok().json(performance))
}

pub async fn train_ai_model(
    path: web::Path<Uuid>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    
    ai_service.train_ai_model(&model_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Model training started successfully"
    })))
}

pub async def deploy_ai_model(
    path: web::Path<Uuid>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    
    ai_service.deploy_ai_model(&model_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Model deployed successfully"
    })))
}

pub async def get_ai_usage_stats(
    ai_repo: web::Data<dyn AIRepository>,
) -> AppResult<HttpResponse> {
    let stats = ai_repo.get_ai_usage_stats().await?;
    
    Ok(HttpResponse::Ok().json(stats))
}

pub async def get_ai_cost_estimate(
    req: web::Json<GenerateContentRequest>,
    ai_service: web::Data<dyn AIService>,
) -> AppResult<HttpResponse> {
    let request_data = req.into_inner();
    
    let estimate = ai_service.get_cost_estimate(
        &request_data.prompt,
        request_data.content_type,
        request_data.model,
        request_data.parameters,
    ).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "estimated_cost": estimate,
        "currency": "USD"
    })))
}