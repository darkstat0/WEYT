use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::user::*;
use crate::repositories::user_repository::*;
use crate::services::user_service::*;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: UserRole,
    pub trust_level: TrustLevel,
    pub is_verified: bool,
    pub is_premium: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPreferencesRequest {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub notifications: Option<NotificationSettings>,
    pub privacy: Option<PrivacySettings>,
    pub parental_controls: Option<ParentalControls>,
}

#[derive(Debug, Deserialize)]
pub struct SearchUsersQuery {
    pub query: String,
    pub role: Option<UserRole>,
    pub trust_level: Option<TrustLevel>,
    pub is_verified: Option<bool>,
    pub is_premium: Option<bool>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub async fn get_users(
    query: web::Query<SearchUsersQuery>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = user_repo.search_users(
        &query.query,
        query.role.as_ref(),
        query.trust_level.as_ref(),
        query.is_verified,
        query.is_premium,
        page,
        page_size,
    ).await?;
    
    let response = UserListResponse {
        users: result.users.into_iter().map(|user| UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            role: user.role,
            trust_level: user.trust_level,
            is_verified: user.is_verified,
            is_premium: user.is_premium,
            created_at: user.created_at,
        }).collect(),
        total: result.total,
        page,
        page_size,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_user(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let user = user_repo.get_user_by_id(&user_id).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_user(
    req: web::Json<CreateUserRequest>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_data = req.into_inner();
    
    let user = user_service.create_user(
        &user_data.username,
        &user_data.email,
        &user_data.password,
        user_data.full_name,
    ).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Created().json(response))
}

pub async fn update_user(
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let update_data = req.into_inner();
    
    let user = user_service.update_user(
        &user_id,
        update_data.full_name,
        update_data.bio,
        update_data.avatar_url,
    ).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_user(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    user_service.delete_user(&user_id).await?;
    
    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_user_profile(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let profile = user_repo.get_user_profile(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(profile))
}

pub async fn update_user_profile(
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let update_data = req.into_inner();
    
    let user = user_service.update_user_profile(
        &user_id,
        update_data.full_name,
        update_data.bio,
        update_data.avatar_url,
    ).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_user_videos(
    path: web::Path<Uuid>,
    query: web::Query<SearchUsersQuery>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let result = user_repo.get_user_videos(&user_id, &query.query, page, page_size).await?;
    
    Ok(HttpResponse::Ok().json(result))
}

pub async fn update_user_preferences(
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserPreferencesRequest>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let preferences_data = req.into_inner();
    
    let user = user_service.update_user_preferences(
        &user_id,
        preferences_data.language,
        preferences_data.theme,
        preferences_data.notifications,
        preferences_data.privacy,
        preferences_data.parental_controls,
    ).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn verify_user_email(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    user_service.verify_user_email(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email verified successfully"
    })))
}

pub async fn verify_user_phone(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    user_service.verify_user_phone(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Phone verified successfully"
    })))
}

pub async fn upgrade_to_premium(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    let user = user_service.upgrade_to_premium(&user_id).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn downgrade_from_premium(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    let user = user_service.downgrade_from_premium(&user_id).await?;
    
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        role: user.role,
        trust_level: user.trust_level,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        created_at: user.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_user_statistics(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let stats = user_repo.get_user_statistics(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn get_user_analytics(
    path: web::Path<Uuid>,
    query: web::Query<SearchUsersQuery>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let analytics = user_repo.get_user_analytics(&user_id, &query.query).await?;
    
    Ok(HttpResponse::Ok().json(analytics))
}

pub async fn get_user_activity(
    path: web::Path<Uuid>,
    query: web::Query<SearchUsersQuery>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let activity = user_repo.get_user_activity(&user_id, &query.query).await?;
    
    Ok(HttpResponse::Ok().json(activity))
}

pub async fn get_user_watch_history(
    path: web::Path<Uuid>,
    query: web::Query<SearchUsersQuery>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let watch_history = user_repo.get_user_watch_history(&user_id, &query.query).await?;
    
    Ok(HttpResponse::Ok().json(watch_history))
}

pub async fn get_user_creator_level(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let level = user_repo.get_user_creator_level(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(level))
}

pub async fn update_user_creator_level(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    let level = user_service.update_user_creator_level(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(level))
}

pub async fn get_user_permissions(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let permissions = user_repo.get_user_permissions(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(permissions))
}

pub async fn grant_user_permissions(
    path: web::Path<Uuid>,
    req: web::Json<Vec<Permission>>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let permissions = req.into_inner();
    
    user_service.grant_user_permissions(&user_id, permissions).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Permissions granted successfully"
    })))
}

pub async fn revoke_user_permissions(
    path: web::Path<Uuid>,
    req: web::Json<Vec<Permission>>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let permissions = req.into_inner();
    
    user_service.revoke_user_permissions(&user_id, permissions).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Permissions revoked successfully"
    })))
}

pub async fn get_user_sessions(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let sessions = user_repo.get_user_sessions(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(sessions))
}

pub async fn revoke_user_session(
    path: web::Path<Uuid>,
    session_id: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let session_id = session_id.into_inner();
    
    user_service.revoke_user_session(&user_id, &session_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Session revoked successfully"
    })))
}

pub async fn revoke_all_user_sessions(
    path: web::Path<Uuid>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    
    user_service.revoke_all_user_sessions(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All sessions revoked successfully"
    })))
}

pub async fn get_user_security_settings(
    path: web::Path<Uuid>,
    user_repo: web::Data<dyn UserRepository>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let settings = user_repo.get_user_security_settings(&user_id).await?;
    
    Ok(HttpResponse::Ok().json(settings))
}

pub async fn update_user_security_settings(
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserSecuritySettingsRequest>,
    user_service: web::Data<dyn UserService>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let settings = req.into_inner();
    
    user_service.update_user_security_settings(&user_id, settings).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Security settings updated successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSecuritySettingsRequest {
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
    pub login_notifications_enabled: Option<bool>,
    pub trusted_devices: Option<Vec<String>>,
    pub password_policy: Option<PasswordPolicy>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_numbers: Option<bool>,
    pub require_special_chars: Option<bool>,
    pub expire_days: Option<i32>,
}