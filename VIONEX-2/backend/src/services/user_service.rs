use uuid::Uuid;
use chrono::{DateTime, Utc};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use crate::models::user::*;
use crate::repositories::user_repository::UserRepository;
use crate::utils::error::{AppError, AppResult};

#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        full_name: Option<String>,
    ) -> Result<User, AppError>;
    
    async fn update_user(
        &self,
        user_id: &Uuid,
        full_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, AppError>;
    
    async fn update_user_profile(
        &self,
        user_id: &Uuid,
        full_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, AppError>;
    
    async fn update_user_preferences(
        &self,
        user_id: &Uuid,
        language: Option<String>,
        theme: Option<String>,
        notifications: Option<NotificationSettings>,
        privacy: Option<PrivacySettings>,
        parental_controls: Option<ParentalControls>,
    ) -> Result<User, AppError>;
    
    async fn verify_user_email(&self, user_id: &Uuid) -> Result<(), AppError>;
    async fn verify_user_phone(&self, user_id: &Uuid) -> Result<(), AppError>;
    async fn upgrade_to_premium(&self, user_id: &Uuid) -> Result<User, AppError>;
    async fn downgrade_from_premium(&self, user_id: &Uuid) -> Result<User, AppError>;
    async fn update_user_creator_level(&self, user_id: &Uuid) -> Result<CreatorLevel, AppError>;
    async fn grant_user_permissions(&self, user_id: &Uuid, permissions: Vec<Permission>) -> Result<(), AppError>;
    async fn revoke_user_permissions(&self, user_id: &Uuid, permissions: Vec<Permission>) -> Result<(), AppError>;
    async fn revoke_user_session(&self, user_id: &Uuid, session_id: &Uuid) -> Result<(), AppError>;
    async fn revoke_all_user_sessions(&self, user_id: &Uuid) -> Result<(), AppError>;
    async fn update_user_security_settings(&self, user_id: &Uuid, settings: UpdateUserSecuritySettingsRequest) -> Result<(), AppError>;
}

pub struct UserServiceImpl {
    user_repository: Box<dyn UserRepository>,
    jwt_secret: String,
}

impl UserServiceImpl {
    pub fn new(user_repository: Box<dyn UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repository,
            jwt_secret,
        }
    }
    
    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        verify(password, hash)
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    
    fn generate_jwt(&self, user: &User) -> Result<String, AppError> {
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }
    
    fn generate_refresh_token(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        full_name: Option<String>,
    ) -> Result<User, AppError> {
        // Check if user already exists
        if let Ok(_) = self.user_repository.get_user_by_email(email).await {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }
        
        if let Ok(_) = self.user_repository.get_user_by_username(username).await {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }
        
        // Hash password
        let password_hash = self.hash_password(password)?;
        
        // Create user
        let mut user = User::new(
            username.to_string(),
            email.to_string(),
            password_hash,
            UserRole::Viewer,
        );
        
        user.full_name = full_name;
        user.created_at = Utc::now();
        user.updated_at = Utc::now();
        
        let created_user = self.user_repository.create_user(&user).await?;
        
        // Generate initial creator level
        self.update_user_creator_level(&created_user.id).await?;
        
        Ok(created_user)
    }
    
    async fn update_user(
        &self,
        user_id: &Uuid,
        full_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        user.full_name = full_name;
        user.bio = bio;
        user.avatar_url = avatar_url;
        user.updated_at = Utc::now();
        
        self.user_repository.update_user(user_id, &user).await
    }
    
    async fn update_user_profile(
        &self,
        user_id: &Uuid,
        full_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User, AppError> {
        self.update_user(user_id, full_name, bio, avatar_url).await
    }
    
    async fn update_user_preferences(
        &self,
        user_id: &Uuid,
        language: Option<String>,
        theme: Option<String>,
        notifications: Option<NotificationSettings>,
        privacy: Option<PrivacySettings>,
        parental_controls: Option<ParentalControls>,
    ) -> Result<User, AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        if let Some(language) = language {
            user.preferences.language = language;
        }
        
        if let Some(theme) = theme {
            user.preferences.theme = theme;
        }
        
        if let Some(notifications) = notifications {
            user.preferences.notifications = notifications;
        }
        
        if let Some(privacy) = privacy {
            user.preferences.privacy = privacy;
        }
        
        if let Some(parental_controls) = parental_controls {
            user.preferences.parental_controls = parental_controls;
        }
        
        user.updated_at = Utc::now();
        
        self.user_repository.update_user(user_id, &user).await
    }
    
    async fn verify_user_email(&self, user_id: &Uuid) -> Result<(), AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        user.email_verified = true;
        user.updated_at = Utc::now();
        
        self.user_repository.update_user(user_id, &user).await?;
        
        // Log verification event
        self.log_user_activity(user_id, "email_verified", serde_json::json!({})).await?;
        
        Ok(())
    }
    
    async fn verify_user_phone(&self, user_id: &Uuid) -> Result<(), AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        user.phone_verified = true;
        user.updated_at = Utc::now();
        
        self.user_repository.update_user(user_id, &user).await?;
        
        // Log verification event
        self.log_user_activity(user_id, "phone_verified", serde_json::json!({})).await?;
        
        Ok(())
    }
    
    async fn upgrade_to_premium(&self, user_id: &Uuid) -> Result<User, AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        user.is_premium = true;
        user.updated_at = Utc::now();
        
        // Update trust level
        if user.trust_level == TrustLevel::Medium {
            user.trust_level = TrustLevel::High;
        } else if user.trust_level == TrustLevel::High {
            user.trust_level = TrustLevel::Platinum;
        }
        
        let updated_user = self.user_repository.update_user(user_id, &user).await?;
        
        // Log upgrade event
        self.log_user_activity(user_id, "upgraded_to_premium", serde_json::json!({
            "previous_level": user.trust_level,
            "new_level": updated_user.trust_level
        })).await?;
        
        Ok(updated_user)
    }
    
    async fn downgrade_from_premium(&self, user_id: &Uuid) -> Result<User, AppError> {
        let mut user = self.user_repository.get_user_by_id(user_id).await?;
        
        user.is_premium = false;
        user.updated_at = Utc::now();
        
        // Downgrade trust level
        if user.trust_level == TrustLevel::Platinum {
            user.trust_level = TrustLevel::High;
        } else if user.trust_level == TrustLevel::High {
            user.trust_level = TrustLevel::Medium;
        }
        
        let updated_user = self.user_repository.update_user(user_id, &user).await?;
        
        // Log downgrade event
        self.log_user_activity(user_id, "downgraded_from_premium", serde_json::json!({
            "previous_level": user.trust_level,
            "new_level": updated_user.trust_level
        })).await?;
        
        Ok(updated_user)
    }
    
    async fn update_user_creator_level(&self, user_id: &Uuid) -> Result<CreatorLevel, AppError> {
        let user = self.user_repository.get_user_by_id(user_id).await?;
        let stats = self.user_repository.get_user_statistics(user_id).await?;
        
        // Calculate experience based on views, likes, and videos
        let mut experience = stats.total_views / 1000; // 1 point per 1000 views
        experience += stats.total_likes * 10; // 10 points per like
        experience += stats.total_videos * 100; // 100 points per video
        
        // Determine level
        let level = if experience < 1000 {
            1
        } else if experience < 5000 {
            2
        } else if experience < 20000 {
            3
        } else if experience < 100000 {
            4
        } else if experience < 500000 {
            5
        } else if experience < 2000000 {
            6
        } else if experience < 10000000 {
            7
        } else {
            8
        };
        
        let next_level_experience = match level {
            1 => 1000,
            2 => 5000,
            3 => 20000,
            4 => 100000,
            5 => 500000,
            6 => 2000000,
            7 => 10000000,
            _ => i64::MAX,
        };
        
        // Determine badges
        let mut badges = Vec::new();
        
        if stats.total_videos >= 10 {
            badges.push("Video Creator".to_string());
        }
        if stats.total_videos >= 100 {
            badges.push("Power Creator".to_string());
        }
        if stats.total_videos >= 1000 {
            badges.push("Mega Creator".to_string());
        }
        
        if stats.total_views >= 100000 {
            badges.push("Trending".to_string());
        }
        if stats.total_views >= 1000000 {
            badges.push("Viral".to_string());
        }
        if stats.total_views >= 10000000 {
            badges.push("Superstar".to_string());
        }
        
        if stats.total_likes >= 10000 {
            badges.push("Loved".to_string());
        }
        if stats.total_likes >= 100000 {
            badges.push("Adored".to_string());
        }
        
        // Determine unlocked features
        let mut unlocked_features = Vec::new();
        
        if level >= 2 {
            unlocked_features.push("Basic Analytics".to_string());
        }
        if level >= 3 {
            unlocked_features.push("Custom Thumbnails".to_string());
        }
        if level >= 4 {
            unlocked_features.push("Live Streaming".to_string());
        }
        if level >= 5 {
            unlocked_features.push("Monetization".to_string());
        }
        if level >= 6 {
            unlocked_features.push("Advanced Analytics".to_string());
        }
        if level >= 7 {
            unlocked_features.push("Priority Support".to_string());
        }
        if level >= 8 {
            unlocked_features.push("Early Access Features".to_string());
        }
        
        let creator_level = CreatorLevel {
            level,
            experience,
            next_level_experience,
            badges,
            unlocked_features,
        };
        
        // Log level update
        self.log_user_activity(user_id, "creator_level_updated", serde_json::json!({
            "level": level,
            "experience": experience,
            "badges": badges,
            "unlocked_features": unlocked_features
        })).await?;
        
        Ok(creator_level)
    }
    
    async fn grant_user_permissions(&self, user_id: &Uuid, permissions: Vec<Permission>) -> Result<(), AppError> {
        for permission in permissions {
            // Check if user already has this permission
            let existing_permissions = self.user_repository.get_user_permissions(user_id).await?;
            if !existing_permissions.contains(&permission) {
                // Grant permission
                sqlx::query!(
                    "INSERT INTO user_permissions (user_id, permission) VALUES ($1, $2)",
                    user_id,
                    permission.to_string()
                )
                .execute(&self.user_repository.get_pool())
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
        
        // Log permission grant
        self.log_user_activity(user_id, "permissions_granted", serde_json::json!({
            "permissions": permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>()
        })).await?;
        
        Ok(())
    }
    
    async fn revoke_user_permissions(&self, user_id: &Uuid, permissions: Vec<Permission>) -> Result<(), AppError> {
        for permission in permissions {
            sqlx::query!(
                "DELETE FROM user_permissions WHERE user_id = $1 AND permission = $2",
                user_id,
                permission.to_string()
            )
            .execute(&self.user_repository.get_pool())
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }
        
        // Log permission revoke
        self.log_user_activity(user_id, "permissions_revoked", serde_json::json!({
            "permissions": permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>()
        })).await?;
        
        Ok(())
    }
    
    async fn revoke_user_session(&self, user_id: &Uuid, session_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM user_sessions WHERE user_id = $1 AND id = $2",
            user_id,
            session_id
        )
        .execute(&self.user_repository.get_pool())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        // Log session revoke
        self.log_user_activity(user_id, "session_revoked", serde_json::json!({
            "session_id": session_id
        })).await?;
        
        Ok(())
    }
    
    async fn revoke_all_user_sessions(&self, user_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM user_sessions WHERE user_id = $1",
            user_id
        )
        .execute(&self.user_repository.get_pool())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        // Log session revoke
        self.log_user_activity(user_id, "all_sessions_revoked", serde_json::json!({})).await?;
        
        Ok(())
    }
    
    async fn update_user_security_settings(&self, user_id: &Uuid, settings: UpdateUserSecuritySettingsRequest) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO user_security_settings (
                user_id, two_factor_enabled, two_factor_secret,
                login_notifications_enabled, trusted_devices, password_policy
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) DO UPDATE SET
                two_factor_enabled = $2,
                two_factor_secret = $3,
                login_notifications_enabled = $4,
                trusted_devices = $5,
                password_policy = $6
            "#,
            user_id,
            settings.two_factor_enabled,
            settings.two_factor_secret,
            settings.login_notifications_enabled,
            serde_json::to_value(settings.trusted_devices).unwrap_or_default(),
            serde_json::to_value(settings.password_policy).unwrap_or_default()
        )
        .execute(&self.user_repository.get_pool())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        // Log security settings update
        self.log_user_activity(user_id, "security_settings_updated", serde_json::json!({
            "two_factor_enabled": settings.two_factor_enabled,
            "login_notifications_enabled": settings.login_notifications_enabled
        })).await?;
        
        Ok(())
    }
    
    async fn log_user_activity(&self, user_id: &Uuid, activity_type: &str, activity_data: serde_json::Value) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO user_activities (user_id, activity_type, activity_data) VALUES ($1, $2, $3)",
            user_id,
            activity_type,
            activity_data
        )
        .execute(&self.user_repository.get_pool())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    email: String,
    role: UserRole,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSecuritySettingsRequest {
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
    pub login_notifications_enabled: Option<bool>,
    pub trusted_devices: Option<Vec<String>>,
    pub password_policy: Option<PasswordPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_numbers: Option<bool>,
    pub require_special_chars: Option<bool>,
    pub expire_days: Option<i32>,
}