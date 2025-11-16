use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: UserRole,
    pub trust_level: TrustLevel,
    pub is_verified: bool,
    pub is_premium: bool,
    pub is_active: bool,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "user_role")]
pub enum UserRole {
    Viewer,
    Creator,
    Brand,
    Advertiser,
    Moderator,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "trust_level")]
pub enum TrustLevel {
    Low,
    Medium,
    High,
    Platinum,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub language: String,
    pub theme: String,
    pub notifications: NotificationSettings,
    pub privacy: PrivacySettings,
    pub parental_controls: ParentalControls,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email: bool,
    pub push: bool,
    pub sms: bool,
    pub marketing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_visibility: ProfileVisibility,
    pub activity_visibility: ActivityVisibility,
    pub data_sharing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProfileVisibility {
    Public,
    Friends,
    Private,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActivityVisibility {
    Public,
    Friends,
    Private,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentalControls {
    pub enabled: bool,
    pub age_restriction: AgeRestriction,
    pub content_filter: ContentFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgeRestriction {
    None,
    Teen,
    Adult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentFilter {
    pub violence: bool,
    pub adult: bool,
    pub hate_speech: bool,
    pub explicit: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdate {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: UserRole,
    pub trust_level: TrustLevel,
    pub is_verified: bool,
    pub is_premium: bool,
    pub created_at: DateTime<Utc>,
    pub statistics: UserStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStatistics {
    pub total_videos: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_subscribers: i64,
    pub join_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerification {
    pub email: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub enabled: bool,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub total_watch_time: i64,
    pub videos_watched: i64,
    pub likes_given: i64,
    pub comments_posted: i64,
    pub shares_count: i64,
    pub favorite_videos: Vec<Uuid>,
    pub watch_history: Vec<WatchHistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchHistoryEntry {
    pub video_id: Uuid,
    pub watched_at: DateTime<Utc>,
    pub watch_duration: i64,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorLevel {
    pub level: i32,
    pub experience: i64,
    pub next_level_experience: i64,
    pub badges: Vec<String>,
    pub unlocked_features: Vec<String>,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            full_name: None,
            avatar_url: None,
            bio: None,
            role,
            trust_level: TrustLevel::Medium,
            is_verified: false,
            is_premium: false,
            is_active: true,
            email_verified: false,
            phone_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            preferences: UserPreferences {
                language: "en".to_string(),
                theme: "light".to_string(),
                notifications: NotificationSettings {
                    email: true,
                    push: true,
                    sms: false,
                    marketing: false,
                },
                privacy: PrivacySettings {
                    profile_visibility: ProfileVisibility::Public,
                    activity_visibility: ActivityVisibility::Friends,
                    data_sharing: false,
                },
                parental_controls: ParentalControls {
                    enabled: false,
                    age_restriction: AgeRestriction::None,
                    content_filter: ContentFilter {
                        violence: true,
                        adult: true,
                        hate_speech: true,
                        explicit: true,
                    },
                },
            },
        }
    }

    pub fn is_creator(&self) -> bool {
        matches!(self.role, UserRole::Creator)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    pub fn can_upload_videos(&self) -> bool {
        matches!(self.role, UserRole::Creator | UserRole::Brand | UserRole::Admin)
    }

    pub fn can_monetize(&self) -> bool {
        matches!(self.role, UserRole::Creator | UserRole::Brand)
    }
}

impl TrustLevel {
    pub fn numeric_value(&self) -> i32 {
        match self {
            TrustLevel::Low => 1,
            TrustLevel::Medium => 2,
            TrustLevel::High => 3,
            TrustLevel::Platinum => 4,
        }
    }

    pub fn from_numeric(value: i32) -> Self {
        match value {
            1 => TrustLevel::Low,
            2 => TrustLevel::Medium,
            3 => TrustLevel::High,
            4 => TrustLevel::Platinum,
            _ => TrustLevel::Medium,
        }
    }
}

impl UserRole {
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            UserRole::Viewer => vec![
                Permission::ViewVideos,
                Permission::LikeVideos,
                Permission::Comment,
                Permission::ShareVideos,
                Permission::SaveVideos,
            ],
            UserRole::Creator => vec![
                Permission::ViewVideos,
                Permission::LikeVideos,
                Permission::Comment,
                Permission::ShareVideos,
                Permission::SaveVideos,
                Permission::UploadVideos,
                Permission::EditOwnVideos,
                Permission::DeleteOwnVideos,
                Permission::MonetizeVideos,
                Permission::AccessCreatorStudio,
            ],
            UserRole::Brand => vec![
                Permission::ViewVideos,
                Permission::LikeVideos,
                Permission::Comment,
                Permission::ShareVideos,
                Permission::SaveVideos,
                Permission::UploadVideos,
                Permission::EditOwnVideos,
                Permission::DeleteOwnVideos,
                Permission::MonetizeVideos,
                Permission::AccessCreatorStudio,
                Permission::CreateBrandChannels,
            ],
            UserRole::Advertiser => vec![
                Permission::ViewVideos,
                Permission::CreateAdCampaigns,
                Permission::ViewAdAnalytics,
                Permission::ManageAdBudget,
            ],
            UserRole::Moderator => vec![
                Permission::ViewVideos,
                Permission::ModerateContent,
                Permission::ViewReports,
                Permission::BanUsers,
                Permission::DeleteVideos,
            ],
            UserRole::Admin => vec![
                Permission::ViewVideos,
                Permission::LikeVideos,
                Permission::Comment,
                Permission::ShareVideos,
                Permission::SaveVideos,
                Permission::UploadVideos,
                Permission::EditAnyVideos,
                Permission::DeleteAnyVideos,
                Permission::MonetizeVideos,
                Permission::AccessCreatorStudio,
                Permission::ModerateContent,
                Permission::ViewReports,
                Permission::BanUsers,
                Permission::ManageUsers,
                Permission::ManageSystem,
                Permission::ViewAnalytics,
                Permission::ManageRevenue,
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Permission {
    ViewVideos,
    LikeVideos,
    Comment,
    ShareVideos,
    SaveVideos,
    UploadVideos,
    EditOwnVideos,
    EditAnyVideos,
    DeleteOwnVideos,
    DeleteAnyVideos,
    MonetizeVideos,
    AccessCreatorStudio,
    CreateBrandChannels,
    CreateAdCampaigns,
    ViewAdAnalytics,
    ManageAdBudget,
    ModerateContent,
    ViewReports,
    BanUsers,
    ManageUsers,
    ManageSystem,
    ViewAnalytics,
    ManageRevenue,
}