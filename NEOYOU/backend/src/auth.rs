use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, BcryptError};
use std::env;
use crate::config::Config;
use crate::database::DbPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub username: String,
    pub role: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub is_verified: bool,
    pub is_premium: bool,
    pub trust_level: i32,
    pub role: String,
    pub settings: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

// JWT token generation
pub fn generate_token(user: &User, config: &Config) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let issued_at = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        exp: expiration,
        iat: issued_at,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.security.jwt_secret.as_ref()),
    )
}

// JWT token verification
pub fn verify_token(token: &str, config: &Config) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.security.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

// Password hashing
pub fn hash_password(password: &str, cost: u32) -> Result<String, BcryptError> {
    hash(password, cost)
}

// Password verification
pub fn verify_password(password: &str, hash: &str) -> Result<bool, BcryptError> {
    verify(password, hash)
}

// Rate limiting
pub struct RateLimiter {
    redis_client: redis::Client,
    config: RateLimitConfig,
}

pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl RateLimiter {
    pub fn new(redis_client: redis::Client, config: RateLimitConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let now = Utc::now().timestamp();
        let window_start = now - 60; // Last minute
        
        // Remove old entries
        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(key)
            .arg("-inf")
            .arg(window_start)
            .query_async(&mut conn)
            .await?;

        // Get current count
        let count: Option<i64> = redis::cmd("ZCARD")
            .arg(key)
            .query_async(&mut conn)
            .await?;

        let current_count = count.unwrap_or(0);

        if current_count >= self.config.requests_per_minute as i64 {
            return Ok(false);
        }

        // Add current request with timestamp
        let _: () = redis::cmd("ZADD")
            .arg(key)
            .arg(now)
            .arg(now)
            .query_async(&mut conn)
            .await?;

        // Set expiration
        let _: () = redis::cmd("EXPIRE")
            .arg(key)
            .arg(60) // 1 minute
            .query_async(&mut conn)
            .await?;

        Ok(true)
    }
}

// Login attempt tracking
pub struct LoginAttemptTracker {
    redis_client: redis::Client,
    config: LoginAttemptConfig,
}

pub struct LoginAttemptConfig {
    pub max_attempts: u32,
    pub window_minutes: u64,
}

impl LoginAttemptTracker {
    pub fn new(redis_client: redis::Client, config: LoginAttemptConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    pub async fn record_attempt(&self, user_id: &str, success: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let key = format!("login_attempts:{}", user_id);
        let now = Utc::now().timestamp();
        
        if success {
            // Reset on successful login
            let _: () = redis::cmd("DEL").arg(key).query_async(&mut conn).await?;
        } else {
            // Record failed attempt
            let _: () = redis::cmd("ZADD")
                .arg(&key)
                .arg(now)
                .arg(now)
                .query_async(&mut conn)
                .await?;

            // Remove old attempts
            let window_start = now - (self.config.window_minutes * 60) as i64;
            let _: () = redis::cmd("ZREMRANGEBYSCORE")
                .arg(&key)
                .arg("-inf")
                .arg(window_start)
                .query_async(&mut conn)
                .await?;

            // Set expiration
            let _: () = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(self.config.window_minutes * 60)
                .query_async(&mut conn)
                .await?;
        }

        Ok(())
    }

    pub async fn is_blocked(&self, user_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let key = format!("login_attempts:{}", user_id);
        let count: Option<i64> = redis::cmd("ZCARD").arg(&key).query_async(&mut conn).await?;
        
        Ok(count.unwrap_or(0) >= self.config.max_attempts as i64)
    }
}

// Auth middleware
pub async fn authenticate_user(
    pool: &DbPool,
    token: &str,
    config: &Config,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    // Verify JWT token
    let claims = verify_token(token, config)?;
    
    // Fetch user from database
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, username, full_name, avatar_url, bio, created_at, updated_at,
               last_login, is_active, is_verified, is_premium, trust_level, role,
               settings, preferences
        FROM users
        WHERE id = $1 AND is_active = TRUE
        "#,
        Uuid::parse_str(&claims.sub)?
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

// Refresh token generation
pub fn refresh_token(user: &User, config: &Config) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7)) // 7 days refresh token
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let issued_at = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        exp: expiration,
        iat: issued_at,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.security.jwt_secret.as_ref()),
    )
}

// Password reset token generation
pub fn generate_password_reset_token(user_id: &Uuid) -> String {
    format!("reset:{}", user_id)
}

// Password reset token verification
pub fn verify_password_reset_token(token: &str) -> Option<Uuid> {
    if let Some(id_str) = token.strip_prefix("reset:") {
        Uuid::parse_str(id_str).ok()
    } else {
        None
    }
}

// Two-factor authentication helpers
pub struct TwoFactorAuth;

impl TwoFactorAuth {
    pub fn generate_totp_secret() -> String {
        // In production, use a proper TOTP library like 'totp-lite'
        // For now, return a placeholder
        "placeholder-secret".to_string()
    }

    pub fn verify_totp_code(secret: &str, code: &str) -> bool {
        // In production, implement proper TOTP verification
        // For now, accept any 6-digit code
        code.len() == 6 && code.chars().all(|c| c.is_ascii_digit())
    }
}

// Session management
pub struct SessionManager {
    redis_client: redis::Client,
}

impl SessionManager {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    pub async fn create_session(&self, user_id: &Uuid, token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let key = format!("session:{}", user_id);
        let expiration = 86400; // 24 hours
        
        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(expiration)
            .arg(token)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn verify_session(&self, user_id: &Uuid, token: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let key = format!("session:{}", user_id);
        let stored_token: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;
        
        Ok(stored_token.map_or(false, |t| t == token))
    }

    pub async fn invalidate_session(&self, user_id: &Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let key = format!("session:{}", user_id);
        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
        
        Ok(())
    }
}