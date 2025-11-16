use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::user::*;
use crate::services::auth_service::*;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

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

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorSetupRequest {
    pub enabled: bool,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EmailVerificationResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionResponse>,
    pub total: i64,
}

pub async fn login(
    req: web::Json<LoginRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let login_data = req.into_inner();
    
    let auth_result = auth_service.login(
        &login_data.email,
        &login_data.password,
        login_data.remember_me,
    ).await?;
    
    let response = AuthResponse {
        user: UserResponse {
            id: auth_result.user.id,
            username: auth_result.user.username,
            email: auth_result.user.email,
            full_name: auth_result.user.full_name,
            avatar_url: auth_result.user.avatar_url,
            bio: auth_result.user.bio,
            role: auth_result.user.role,
            trust_level: auth_result.user.trust_level,
            is_verified: auth_result.user.is_verified,
            is_premium: auth_result.user.is_premium,
            created_at: auth_result.user.created_at,
        },
        access_token: auth_result.access_token,
        refresh_token: auth_result.refresh_token,
        expires_in: auth_result.expires_in,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn register(
    req: web::Json<RegisterRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let register_data = req.into_inner();
    
    let user = auth_service.register(
        &register_data.username,
        &register_data.email,
        &register_data.password,
        register_data.full_name,
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

pub async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let refresh_data = req.into_inner();
    
    let auth_result = auth_service.refresh_token(&refresh_data.refresh_token).await?;
    
    let response = AuthResponse {
        user: UserResponse {
            id: auth_result.user.id,
            username: auth_result.user.username,
            email: auth_result.user.email,
            full_name: auth_result.user.full_name,
            avatar_url: auth_result.user.avatar_url,
            bio: auth_result.user.bio,
            role: auth_result.user.role,
            trust_level: auth_result.user.trust_level,
            is_verified: auth_result.user.is_verified,
            is_premium: auth_result.user.is_premium,
            created_at: auth_result.user.created_at,
        },
        access_token: auth_result.access_token,
        refresh_token: auth_result.refresh_token,
        expires_in: auth_result.expires_in,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn logout(
    req: web::Json<RefreshTokenRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let refresh_data = req.into_inner();
    
    auth_service.logout(&refresh_data.refresh_token).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

pub async fn get_current_user(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let user = auth_service.get_current_user().await?;
    
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

pub async fn forgot_password(
    req: web::Json<ForgotPasswordRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let forgot_data = req.into_inner();
    
    let reset_response = auth_service.forgot_password(&forgot_data.email).await?;
    
    Ok(HttpResponse::Ok().json(reset_response))
}

pub async fn reset_password(
    req: web::Json<ResetPasswordRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let reset_data = req.into_inner();
    
    auth_service.reset_password(&reset_data.token, &reset_data.new_password).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset successfully"
    })))
}

pub async fn verify_email(
    req: web::Json<VerifyEmailRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let verify_data = req.into_inner();
    
    auth_service.verify_email(&verify_data.token).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email verified successfully"
    })))
}

pub async fn setup_two_factor(
    req: web::Json<TwoFactorSetupRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let setup_data = req.into_inner();
    
    let setup_response = auth_service.setup_two_factor(setup_data.enabled, setup_data.secret).await?;
    
    Ok(HttpResponse::Ok().json(setup_response))
}

pub async fn verify_two_factor(
    req: web::Json<TwoFactorVerifyRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let verify_data = req.into_inner();
    
    auth_service.verify_two_factor(&verify_data.code).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Two-factor authentication verified successfully"
    })))
}

pub async fn get_user_sessions(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let sessions = auth_service.get_user_sessions().await?;
    
    let response = SessionListResponse {
        sessions: sessions.into_iter().map(|session| SessionResponse {
            id: session.id,
            device_info: session.device_info,
            ip_address: session.ip_address,
            created_at: session.created_at,
            last_used_at: session.last_used_at,
            expires_at: session.expires_at,
        }).collect(),
        total: sessions.len() as i64,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub async fn revoke_session(
    session_id: web::Path<Uuid>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let session_id = session_id.into_inner();
    
    auth_service.revoke_session(&session_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Session revoked successfully"
    })))
}

pub async fn revoke_all_sessions(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    auth_service.revoke_all_sessions().await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All sessions revoked successfully"
    })))
}

pub async fn change_password(
    req: web::Json<ChangePasswordRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let change_data = req.into_inner();
    
    auth_service.change_password(
        &change_data.current_password,
        &change_data.new_password,
    ).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async def update_profile(
    req: web::Json<UpdateProfileRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let update_data = req.into_inner();
    
    let user = auth_service.update_profile(
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

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

pub async def update_preferences(
    req: web::Json<UpdatePreferencesRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let preferences_data = req.into_inner();
    
    auth_service.update_preferences(
        preferences_data.language,
        preferences_data.theme,
        preferences_data.notifications,
        preferences_data.privacy,
        preferences_data.parental_controls,
    ).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Preferences updated successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub notifications: Option<NotificationSettings>,
    pub privacy: Option<PrivacySettings>,
    pub parental_controls: Option<ParentalControls>,
}

pub async def get_security_settings(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let settings = auth_service.get_security_settings().await?;
    
    Ok(HttpResponse::Ok().json(settings))
}

pub async def update_security_settings(
    req: web::Json<UpdateSecuritySettingsRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let settings_data = req.into_inner();
    
    auth_service.update_security_settings(settings_data).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Security settings updated successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecuritySettingsRequest {
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
    pub login_notifications_enabled: Option<bool>,
    pub trusted_devices: Option<Vec<String>>,
    pub password_policy: Option<PasswordPolicy>,
}

pub async def get_activity_log(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let activity = auth_service.get_activity_log().await?;
    
    Ok(HttpResponse::Ok().json(activity))
}

pub async def download_data(
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let data = auth_service.download_user_data().await?;
    
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("Content-Disposition", "attachment; filename=user-data.json"))
        .body(data))
}

pub async def delete_account(
    req: web::Json<DeleteAccountRequest>,
    auth_service: web::Data<dyn AuthService>,
) -> AppResult<HttpResponse> {
    let delete_data = req.into_inner();
    
    auth_service.delete_account(&delete_data.password).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Account deleted successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}