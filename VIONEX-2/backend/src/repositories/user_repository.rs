use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::models::user::*;
use crate::utils::error::AppError;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<User, AppError>;
    async fn get_user_by_username(&self, username: &str) -> Result<User, AppError>;
    async fn search_users(
        &self,
        query: &str,
        role: Option<&UserRole>,
        trust_level: Option<&TrustLevel>,
        is_verified: Option<bool>,
        is_premium: Option<bool>,
        page: i32,
        page_size: i32,
    ) -> Result<UserSearchResult, AppError>;
    async fn create_user(&self, user: &User) -> Result<User, AppError>;
    async fn update_user(&self, user_id: &Uuid, user: &User) -> Result<User, AppError>;
    async fn delete_user(&self, user_id: &Uuid) -> Result<(), AppError>;
    async fn get_user_profile(&self, user_id: &Uuid) -> Result<UserProfile, AppError>;
    async fn get_user_videos(&self, user_id: &Uuid, query: &str, page: i32, page_size: i32) -> Result<VideoSearchResult, AppError>;
    async fn get_user_statistics(&self, user_id: &Uuid) -> Result<UserStatistics, AppError>;
    async fn get_user_analytics(&self, user_id: &Uuid, query: &str) -> Result<UserAnalytics, AppError>;
    async fn get_user_activity(&self, user_id: &Uuid, query: &str) -> Result<UserActivity, AppError>;
    async fn get_user_watch_history(&self, user_id: &Uuid, query: &str) -> Result<WatchHistory, AppError>;
    async fn get_user_creator_level(&self, user_id: &Uuid) -> Result<CreatorLevel, AppError>;
    async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<Permission>, AppError>;
    async fn get_user_sessions(&self, user_id: &Uuid) -> Result<Vec<Session>, AppError>;
    async fn get_user_security_settings(&self, user_id: &Uuid) -> Result<UserSecuritySettings, AppError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, is_active, email_verified, phone_verified,
                created_at, updated_at, last_login_at, preferences
            FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, is_active, email_verified, phone_verified,
                created_at, updated_at, last_login_at, preferences
            FROM users 
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, is_active, email_verified, phone_verified,
                created_at, updated_at, last_login_at, preferences
            FROM users 
            WHERE username = $1
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user)
    }

    async fn search_users(
        &self,
        query: &str,
        role: Option<&UserRole>,
        trust_level: Option<&TrustLevel>,
        is_verified: Option<bool>,
        is_premium: Option<bool>,
        page: i32,
        page_size: i32,
    ) -> Result<UserSearchResult, AppError> {
        let offset = (page - 1) * page_size;
        
        let mut sql = r#"
            SELECT 
                id, username, email, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, created_at
            FROM users 
            WHERE (username ILIKE $1 OR email ILIKE $1 OR full_name ILIKE $1)
        "#.to_string();
        
        let mut params: Vec<Box<dyn sqlx::IntoArguments<sqlx::Postgres> + Send>> = vec![Box::new(format!("%{}%", query))];
        let mut param_count = 2;
        
        if let Some(role) = role {
            sql.push_str(&format!(" AND role = ${}", param_count));
            params.push(Box::new(role.clone()));
            param_count += 1;
        }
        
        if let Some(trust_level) = trust_level {
            sql.push_str(&format!(" AND trust_level = ${}", param_count));
            params.push(Box::new(trust_level.clone()));
            param_count += 1;
        }
        
        if let Some(is_verified) = is_verified {
            sql.push_str(&format!(" AND is_verified = ${}", param_count));
            params.push(Box::new(is_verified));
            param_count += 1;
        }
        
        if let Some(is_premium) = is_premium {
            sql.push_str(&format!(" AND is_premium = ${}", param_count));
            params.push(Box::new(is_premium));
            param_count += 1;
        }
        
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count, param_count + 1));
        params.push(Box::new(page_size));
        params.push(Box::new(offset));
        
        let users = sqlx::query_as!(
            UserResponse,
            &sql,
            params[0].as_ref().as_any(),
            params[1].as_ref().as_any(),
            params[2].as_ref().as_any(),
            params[3].as_ref().as_any(),
            params[4].as_ref().as_any(),
            params[5].as_ref().as_any(),
            params[6].as_ref().as_any()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM users 
            WHERE (username ILIKE $1 OR email ILIKE $1 OR full_name ILIKE $1)
            "#,
            format!("%{}%", query)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(UserSearchResult {
            users,
            total,
            page,
            page_size,
        })
    }

    async fn create_user(&self, user: &User) -> Result<User, AppError> {
        let created_user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                id, username, email, password_hash, full_name, avatar_url, bio,
                role, trust_level, is_verified, is_premium, is_active,
                email_verified, phone_verified, created_at, updated_at,
                last_login_at, preferences
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18
            ) RETURNING
                id, username, email, password_hash, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, is_active, email_verified, phone_verified,
                created_at, updated_at, last_login_at, preferences
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.full_name,
            user.avatar_url,
            user.bio,
            user.role,
            user.trust_level,
            user.is_verified,
            user.is_premium,
            user.is_active,
            user.email_verified,
            user.phone_verified,
            user.created_at,
            user.updated_at,
            user.last_login_at,
            serde_json::to_value(&user.preferences).unwrap_or_default()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(created_user)
    }

    async fn update_user(&self, user_id: &Uuid, user: &User) -> Result<User, AppError> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET
                username = $1,
                email = $2,
                password_hash = $3,
                full_name = $4,
                avatar_url = $5,
                bio = $6,
                role = $7,
                trust_level = $8,
                is_verified = $9,
                is_premium = $10,
                is_active = $11,
                email_verified = $12,
                phone_verified = $13,
                updated_at = $14,
                preferences = $15
            WHERE id = $16
            RETURNING
                id, username, email, password_hash, full_name, avatar_url, bio,
                role AS "role: UserRole", trust_level AS "trust_level: TrustLevel",
                is_verified, is_premium, is_active, email_verified, phone_verified,
                created_at, updated_at, last_login_at, preferences
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.full_name,
            user.avatar_url,
            user.bio,
            user.role,
            user.trust_level,
            user.is_verified,
            user.is_premium,
            user.is_active,
            user.email_verified,
            user.phone_verified,
            Utc::now(),
            serde_json::to_value(&user.preferences).unwrap_or_default(),
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(updated_user)
    }

    async fn delete_user(&self, user_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_user_profile(&self, user_id: &Uuid) -> Result<UserProfile, AppError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT 
                u.id, u.username, u.email, u.full_name, u.avatar_url, u.bio,
                u.role AS "role: UserRole", u.trust_level AS "trust_level: TrustLevel",
                u.is_verified, u.is_premium, u.created_at,
                COALESCE(v.total_videos, 0) as total_videos,
                COALESCE(v.total_views, 0) as total_views,
                COALESCE(v.total_likes, 0) as total_likes,
                COALESCE(v.total_subscribers, 0) as total_subscribers,
                u.created_at as join_date
            FROM users u
            LEFT JOIN (
                SELECT 
                    user_id,
                    COUNT(*) as total_videos,
                    COALESCE(SUM(view_count), 0) as total_views,
                    COALESCE(SUM(like_count), 0) as total_likes,
                    COUNT(DISTINCT subscriber_id) as total_subscribers
                FROM videos v
                LEFT JOIN video_subscribers vs ON v.id = vs.video_id
                WHERE v.user_id = $1
                GROUP BY user_id
            ) v ON u.id = v.user_id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(profile)
    }

    async fn get_user_videos(&self, user_id: &Uuid, query: &str, page: i32, page_size: i32) -> Result<VideoSearchResult, AppError> {
        let offset = (page - 1) * page_size;
        
        let videos = sqlx::query_as!(
            Video,
            r#"
            SELECT 
                id, title, description, user_id, file_path, thumbnail_path,
                duration, file_size, format, resolution, bitrate, view_count,
                like_count, comment_count, share_count, status, visibility,
                category, tags, ai_analysis, metadata, created_at, updated_at,
                published_at, scheduled_at
            FROM videos 
            WHERE user_id = $1 AND (title ILIKE $2 OR description ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            format!("%{}%", query),
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM videos 
            WHERE user_id = $1 AND (title ILIKE $2 OR description ILIKE $2)
            "#,
            user_id,
            format!("%{}%", query)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(VideoSearchResult {
            videos,
            total,
            page,
            page_size,
        })
    }

    async fn get_user_statistics(&self, user_id: &Uuid) -> Result<UserStatistics, AppError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            SELECT 
                COALESCE(SUM(view_count), 0) as total_watch_time,
                COUNT(*) as videos_watched,
                COALESCE(SUM(like_count), 0) as likes_given,
                COUNT(DISTINCT comment_id) as comments_posted,
                COUNT(DISTINCT share_id) as shares_count
            FROM videos v
            LEFT JOIN video_likes vl ON v.id = vl.video_id
            LEFT JOIN video_comments vc ON v.id = vc.video_id
            LEFT JOIN video_shares vs ON v.id = vs.video_id
            WHERE v.user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(stats)
    }

    async fn get_user_analytics(&self, user_id: &Uuid, query: &str) -> Result<UserAnalytics, AppError> {
        let analytics = sqlx::query_as!(
            UserAnalytics,
            r#"
            SELECT 
                COALESCE(SUM(view_duration), 0) as total_watch_time,
                COUNT(*) as videos_watched,
                COALESCE(SUM(like_count), 0) as likes_given,
                COUNT(DISTINCT comment_id) as comments_posted,
                COUNT(DISTINCT share_id) as shares_count,
                ARRAY_AGG(video_id) as favorite_videos,
                ARRAY_AGG(
                    ROW(
                        video_id,
                        watched_at,
                        watch_duration,
                        completion_rate
                    ) ORDER BY watched_at DESC
                ) as watch_history
            FROM videos v
            LEFT JOIN video_likes vl ON v.id = vl.video_id
            LEFT JOIN video_comments vc ON v.id = vc.video_id
            LEFT JOIN video_shares vs ON v.id = vs.video_id
            LEFT JOIN user_watch_history uwh ON v.id = uwh.video_id
            WHERE v.user_id = $1 AND (title ILIKE $2 OR description ILIKE $2)
            GROUP BY v.user_id
            "#,
            user_id,
            format!("%{}%", query)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(analytics)
    }

    async fn get_user_activity(&self, user_id: &Uuid, query: &str) -> Result<UserActivity, AppError> {
        let activity = sqlx::query_as!(
            UserActivity,
            r#"
            SELECT 
                activity_type,
                activity_data,
                created_at
            FROM user_activities
            WHERE user_id = $1 AND (activity_data::text ILIKE $2)
            ORDER BY created_at DESC
            LIMIT 50
            "#,
            user_id,
            format!("%{}%", query)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(UserActivity { activities: activity })
    }

    async fn get_user_watch_history(&self, user_id: &Uuid, query: &str) -> Result<WatchHistory, AppError> {
        let history = sqlx::query_as!(
            WatchHistoryEntry,
            r#"
            SELECT 
                video_id,
                watched_at,
                watch_duration,
                completion_rate
            FROM user_watch_history
            WHERE user_id = $1 AND (title ILIKE $2 OR description ILIKE $2)
            ORDER BY watched_at DESC
            LIMIT 100
            "#,
            user_id,
            format!("%{}%", query)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(WatchHistory { entries: history })
    }

    async fn get_user_creator_level(&self, user_id: &Uuid) -> Result<CreatorLevel, AppError> {
        let level = sqlx::query_as!(
            CreatorLevel,
            r#"
            SELECT 
                level,
                experience,
                next_level_experience,
                badges,
                unlocked_features
            FROM creator_levels
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .unwrap_or_else(|| CreatorLevel {
            level: 1,
            experience: 0,
            next_level_experience: 1000,
            badges: Vec::new(),
            unlocked_features: Vec::new(),
        });

        Ok(level)
    }

    async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<Permission>, AppError> {
        let permissions = sqlx::query!(
            r#"
            SELECT permission 
            FROM user_permissions 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .into_iter()
        .map(|row| row.permission.parse().unwrap_or(Permission::ViewVideos))
        .collect();

        Ok(permissions)
    }

    async fn get_user_sessions(&self, user_id: &Uuid) -> Result<Vec<Session>, AppError> {
        let sessions = sqlx::query_as!(
            Session,
            r#"
            SELECT 
                id, user_id, token, refresh_token, expires_at,
                created_at, last_used_at, device_info, ip_address
            FROM user_sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(sessions)
    }

    async fn get_user_security_settings(&self, user_id: &Uuid) -> Result<UserSecuritySettings, AppError> {
        let settings = sqlx::query_as!(
            UserSecuritySettings,
            r#"
            SELECT 
                two_factor_enabled, two_factor_secret,
                login_notifications_enabled, trusted_devices,
                password_policy
            FROM user_security_settings
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .unwrap_or_else(|| UserSecuritySettings {
            two_factor_enabled: false,
            two_factor_secret: None,
            login_notifications_enabled: true,
            trusted_devices: Vec::new(),
            password_policy: PasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
                expire_days: 90,
            },
        });

        Ok(settings)
    }
}

#[derive(Debug)]
pub struct UserSearchResult {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize, FromRow)]
pub struct UserProfile {
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
    pub total_videos: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_subscribers: i64,
    pub join_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserAnalytics {
    pub total_watch_time: i64,
    pub videos_watched: i64,
    pub likes_given: i64,
    pub comments_posted: i64,
    pub shares_count: i64,
    pub favorite_videos: Vec<Uuid>,
    pub watch_history: Vec<WatchHistoryEntry>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct WatchHistoryEntry {
    pub video_id: Uuid,
    pub watched_at: DateTime<Utc>,
    pub watch_duration: i64,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct UserActivity {
    pub activities: Vec<ActivityEntry>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ActivityEntry {
    pub activity_type: String,
    pub activity_data: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WatchHistory {
    pub entries: Vec<WatchHistoryEntry>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserSecuritySettings {
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub login_notifications_enabled: bool,
    pub trusted_devices: Vec<String>,
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub expire_days: i32,
}