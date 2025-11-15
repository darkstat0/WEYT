use actix_web::{web, HttpResponse, Responder, Result};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::{auth, database, utils};
use crate::database::DbPool;
use crate::config::Config;
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub settings: Option<HashMap<String, serde_json::Value>>,
    pub preferences: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_verified: bool,
    pub is_premium: bool,
    pub trust_level: i32,
    pub role: String,
    pub settings: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_verified: bool,
    pub is_premium: bool,
    pub trust_level: i32,
    pub role: String,
    pub settings: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

// User registration
pub async fn register(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(errors) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Validation failed".to_string(),
            errors: Some(errors),
        }));
    }

    // Check if username already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1",
        req.username
    )
    .fetch_optional(pool.as_ref())
    .await;

    if let Ok(Some(_)) = existing_user {
        return Ok(HttpResponse::Conflict().json(utils::ErrorResponse {
            success: false,
            message: "Username already exists".to_string(),
            errors: None,
        }));
    }

    // Check if email already exists
    let existing_email = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        req.email
    )
    .fetch_optional(pool.as_ref())
    .await;

    if let Ok(Some(_)) = existing_email {
        return Ok(HttpResponse::Conflict().json(utils::ErrorResponse {
            success: false,
            message: "Email already exists".to_string(),
            errors: None,
        }));
    }

    // Hash password
    let password_hash = match auth::hash_password(&req.password, config.security.bcrypt_cost) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to hash password".to_string(),
                errors: None,
            }));
        }
    };

    // Create new user
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO users (id, email, username, password_hash, full_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, email, username, full_name, avatar_url, bio, created_at, is_active, is_verified, is_premium, trust_level, role, settings, preferences
        "#,
        user_id,
        req.email,
        req.username,
        password_hash,
        req.full_name,
        now,
        now
    )
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(user_data) => {
            let user = UserResponse {
                id: user_data.id,
                username: user_data.username,
                email: user_data.email,
                full_name: user_data.full_name,
                avatar_url: user_data.avatar_url,
                bio: user_data.bio,
                created_at: user_data.created_at,
                is_active: user_data.is_active,
                is_verified: user_data.is_verified,
                is_premium: user_data.is_premium,
                trust_level: user_data.trust_level,
                role: user_data.role,
                settings: user_data.settings,
                preferences: user_data.preferences,
            };

            Ok(HttpResponse::Created().json(utils::ApiResponse {
                success: true,
                message: "User registered successfully".to_string(),
                data: Some(serde_json::to_value(user).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to create user".to_string(),
                errors: None,
            }))
        }
    }
}

// User login
pub async fn login(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(errors) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Validation failed".to_string(),
            errors: Some(errors),
        }));
    }

    // Find user by username or email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, username, password_hash, full_name, avatar_url, bio, created_at, updated_at,
               last_login, is_active, is_verified, is_premium, trust_level, role, settings, preferences
        FROM users
        WHERE (username = $1 OR email = $1) AND is_active = TRUE
        "#,
        req.username
    )
    .fetch_optional(pool.as_ref())
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().json(utils::ErrorResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                errors: None,
            }));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Database error".to_string(),
                errors: None,
            }));
        }
    };

    // Verify password
    if !auth::verify_password(&req.password, &user.password_hash).unwrap_or(false) {
        // Record failed login attempt
        let tracker = auth::LoginAttemptTracker::new(
            database::init_redis(config.as_ref()).await.unwrap(),
            auth::LoginAttemptConfig {
                max_attempts: config.security.max_login_attempts,
                window_minutes: config.security.login_attempt_window / 60,
            },
        );

        if let Err(e) = tracker.record_attempt(&user.id.to_string(), false).await {
            eprintln!("Failed to record login attempt: {}", e);
        }

        return Ok(HttpResponse::Unauthorized().json(utils::ErrorResponse {
            success: false,
            message: "Invalid credentials".to_string(),
            errors: None,
        }));
    }

    // Check if user is blocked due to too many failed attempts
    let tracker = auth::LoginAttemptTracker::new(
        database::init_redis(config.as_ref()).await.unwrap(),
        auth::LoginAttemptConfig {
            max_attempts: config.security.max_login_attempts,
            window_minutes: config.security.login_attempt_window / 60,
        },
    );

    if tracker.is_blocked(&user.id.to_string()).await.unwrap_or(false) {
        return Ok(HttpResponse::TooManyRequests().json(utils::ErrorResponse {
            success: false,
            message: "Too many failed login attempts. Please try again later.".to_string(),
            errors: None,
        }));
    }

    // Record successful login attempt
    if let Err(e) = tracker.record_attempt(&user.id.to_string(), true).await {
        eprintln!("Failed to record successful login: {}", e);
    }

    // Update last login time
    let now = Utc::now();
    if let Err(e) = sqlx::query!(
        "UPDATE users SET last_login = $1 WHERE id = $2",
        now,
        user.id
    )
    .execute(pool.as_ref())
    .await
    {
        eprintln!("Failed to update last login: {}", e);
    }

    // Generate tokens
    let access_token = match auth::generate_token(&user, config.as_ref()) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Failed to generate token: {}", e);
            return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to generate token".to_string(),
                errors: None,
            }));
        }
    };

    let refresh_token = match auth::refresh_token(&user, config.as_ref()) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Failed to generate refresh token: {}", e);
            return Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to generate refresh token".to_string(),
                errors: None,
            }));
        }
    };

    // Create session
    let session_manager = auth::SessionManager::new(
        database::init_redis(config.as_ref()).await.unwrap(),
    );

    if let Err(e) = session_manager.create_session(&user.id, &refresh_token).await {
        eprintln!("Failed to create session: {}", e);
    }

    let user_response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        created_at: user.created_at,
        is_active: user.is_active,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        trust_level: user.trust_level,
        role: user.role,
        settings: user.settings,
        preferences: user.preferences,
    };

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "Login successful".to_string(),
        data: Some(serde_json::json!({
            "user": user_response,
            "access_token": access_token,
            "refresh_token": refresh_token,
            "token_type": "Bearer",
            "expires_in": 86400 // 24 hours
        })),
    }))
}

// Get user profile
pub async fn get_profile(
    pool: web::Data<DbPool>,
    user: web::Data<User>,
) -> Result<HttpResponse> {
    let user_response = UserResponse {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        full_name: user.full_name.clone(),
        avatar_url: user.avatar_url.clone(),
        bio: user.bio.clone(),
        created_at: user.created_at,
        is_active: user.is_active,
        is_verified: user.is_verified,
        is_premium: user.is_premium,
        trust_level: user.trust_level,
        role: user.role.clone(),
        settings: user.settings.clone(),
        preferences: user.preferences.clone(),
    };

    Ok(HttpResponse::Ok().json(utils::ApiResponse {
        success: true,
        message: "User profile retrieved".to_string(),
        data: Some(serde_json::to_value(user_response).unwrap()),
    }))
}

// Update user profile
pub async fn update_profile(
    pool: web::Data<DbPool>,
    user: web::Data<User>,
    req: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse> {
    let now = Utc::now();

    // Build update query dynamically
    let mut query = String::from("UPDATE users SET updated_at = $1");
    let mut param_count = 2;
    let mut params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec
![Box::new(now)];

    if let Some(full_name) = &req.full_name {
        query.push_str(&format!(", full_name = ${}", param_count));
        param_count += 1;
        params.push(Box::new(full_name.clone()));
    }

    if let Some(bio) = &req.bio {
        query.push_str(&format!(", bio = ${}", param_count));
        param_count += 1;
        params.push(Box::new(bio.clone()));
    }

    if let Some(avatar_url) = &req.avatar_url {
        query.push_str(&format!(", avatar_url = ${}", param_count));
        param_count += 1;
        params.push(Box::new(avatar_url.clone()));
    }

    if let Some(settings) = &req.settings {
        query.push_str(&format!(", settings = ${}", param_count));
        param_count += 1;
        params.push(Box::new(settings.clone()));
    }

    if let Some(preferences) = &req.preferences {
        query.push_str(&format!(", preferences = ${}", param_count));
        param_count += 1;
        params.push(Box::new(preferences.clone()));
    }

    query.push_str(&format!(" WHERE id = ${}", param_count));
    param_count += 1;
    params.push(Box::new(user.id));

    // Execute update
    match sqlx::query(&query)
        .bind(now)
        .bind(&user.id)
        .execute(pool.as_ref())
        .await
    {
        Ok(_) => {
            // Fetch updated user
            let updated_user = sqlx::query_as!(
                User,
                r#"
                SELECT id, email, username, password_hash, full_name, avatar_url, bio, created_at, updated_at,
                       last_login, is_active, is_verified, is_premium, trust_level, role, settings, preferences
                FROM users
                WHERE id = $1
                "#,
                user.id
            )
            .fetch_one(pool.as_ref())
            .await;

            match updated_user {
                Ok(user_data) => {
                    let user_response = UserResponse {
                        id: user_data.id,
                        username: user_data.username,
                        email: user_data.email,
                        full_name: user_data.full_name,
                        avatar_url: user_data.avatar_url,
                        bio: user_data.bio,
                        created_at: user_data.created_at,
                        is_active: user_data.is_active,
                        is_verified: user_data.is_verified,
                        is_premium: user_data.is_premium,
                        trust_level: user_data.trust_level,
                        role: user_data.role,
                        settings: user_data.settings,
                        preferences: user_data.preferences,
                    };

                    Ok(HttpResponse::Ok().json(utils::ApiResponse {
                        success: true,
                        message: "Profile updated successfully".to_string(),
                        data: Some(serde_json::to_value(user_response).unwrap()),
                    }))
                }
                Err(e) => {
                    eprintln!("Failed to fetch updated user: {}", e);
                    Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                        success: false,
                        message: "Failed to fetch updated user".to_string(),
                        errors: None,
                    }))
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to update profile".to_string(),
                errors: None,
            }))
        }
    }
}

// Upload avatar
pub async fn upload_avatar(
    pool: web::Data<DbPool>,
    user: web::Data<User>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let mut avatar_url = None;

    while let Some(mut field) = payload.next().await.transpose().unwrap() {
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition
            .get_filename()
            .unwrap_or("unknown")
            .to_string();

        // Validate file type
        if !filename.ends_with(".jpg") && !filename.ends_with(".jpeg") && !filename.ends_with(".png") {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Only JPG, JPEG, and PNG files are allowed".to_string(),
                errors: None,
            }));
        }

        // Save file to S3 or local storage
        // In production, upload to S3 and get the URL
        // For now, use a placeholder URL
        avatar_url = Some(format!("https://example.com/avatars/{}.jpg", user.id));
    }

    if let Some(avatar_url) = avatar_url {
        // Update user avatar URL in database
        match sqlx::query!(
            "UPDATE users SET avatar_url = $1, updated_at = $2 WHERE id = $3",
            avatar_url,
            Utc::now(),
            user.id
        )
        .execute(pool.as_ref())
        .await
        {
            Ok(_) => Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Avatar uploaded successfully".to_string(),
                data: Some(serde_json::json!({"avatar_url": avatar_url})),
            })),
            Err(e) => {
                eprintln!("Database error: {}", e);
                Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                    success: false,
                    message: "Failed to update avatar".to_string(),
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