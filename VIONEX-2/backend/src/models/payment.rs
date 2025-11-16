use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Monetization {
    pub id: Uuid,
    pub user_id: Uuid,
    pub monetization_type: MonetizationType,
    pub status: MonetizationStatus,
    pub settings: MonetizationSettings,
    pub revenue: RevenueMetrics,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "monetization_type")]
pub enum MonetizationType {
    Ads,
    Premium,
    Membership,
    Tips,
    SuperChat,
    BrandDeals,
    Merchandise,
    NFTs,
    Courses,
    ApiAccess,
    VREvents,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "monetization_status")]
pub enum MonetizationStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Terminated,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MonetizationSettings {
    pub ad_settings: Option<AdSettings>,
    pub premium_settings: Option<PremiumSettings>,
    pub membership_settings: Option<MembershipSettings>,
    pub tip_settings: Option<TipSettings>,
    pub brand_deal_settings: Option<BrandDealSettings>,
    pub merchandise_settings: Option<MerchandiseSettings>,
    pub nft_settings: Option<NFTSettings>,
    pub course_settings: Option<CourseSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdSettings {
    pub enabled: bool,
    pub ad_types: Vec<AdType>,
    pub ad_frequency: i32,
    pub ad_break_duration: i32,
    pub skippable_ads: bool,
    pub overlay_ads: bool,
    pub bumper_ads: bool,
    pub mid_roll_ads: bool,
    pub pre_roll_ads: bool,
    pub post_roll_ads: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PremiumSettings {
    pub enabled: bool,
    pub monthly_price: f64,
    pub yearly_price: f64,
    pub features: Vec<String>,
    pub free_trial_days: i32,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MembershipSettings {
    pub enabled: bool,
    pub tiers: Vec<MembershipTier>,
    pub benefits: Vec<String>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MembershipTier {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub benefits: Vec<String>,
    pub monthly_limit: Option<i32>,
    pub custom_badge: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TipSettings {
    pub enabled: bool,
    pub currency: String,
    pub min_amount: f64,
    pub max_amount: f64,
    pub platform_fee_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrandDealSettings {
    pub enabled: bool,
    pub approval_required: bool,
    pub minimum_followers: i64,
    pub engagement_rate_threshold: f64,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchandiseSettings {
    pub enabled: bool,
    pub store_name: String,
    pub categories: Vec<String>,
    pub shipping_zones: Vec<String>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTSettings {
    pub enabled: bool,
    pub platform_fee_percentage: f64,
    pub royalty_percentage: f64,
    pub supported_chains: Vec<String>,
    pub minting_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseSettings {
    pub enabled: bool,
    pub pricing_model: PricingModel,
    pub currency: String,
    pub certificate_enabled: bool,
    pub quiz_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "ad_type")]
pub enum AdType {
    PreRoll,
    MidRoll,
    PostRoll,
    Overlay,
    Bumper,
    Skippable,
    NonSkippable,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "pricing_model")]
pub enum PricingModel {
    OneTime,
    Subscription,
    Free,
    PayWhatYouWant,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RevenueMetrics {
    pub total_revenue: f64,
    pub monthly_revenue: f64,
    pub yearly_revenue: f64,
    pub revenue_by_source: Vec<RevenueBySource>,
    pub payout_amount: f64,
    pub payout_currency: String,
    pub last_payout_date: Option<DateTime<Utc>>,
    pub next_payout_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueBySource {
    pub source: String,
    pub amount: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub payment_type: PaymentType,
    pub amount: f64,
    pub currency: String,
    pub status: PaymentStatus,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub refund_amount: Option<f64>,
    pub refund_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "payment_type")]
pub enum PaymentType {
    Subscription,
    OneTime,
    Donation,
    Tip,
    SuperChat,
    Merchandise,
    Course,
    NFT,
    AdRevenue,
    BrandDeal,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "payment_status")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CreatorPayout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub payout_method: PayoutMethod,
    pub status: PayoutStatus,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "payout_method")]
pub enum PayoutMethod {
    BankTransfer,
    PayPal,
    Stripe,
    Crypto,
    MobileMoney,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "payout_status")]
pub enum PayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdCampaign {
    pub id: Uuid,
    pub advertiser_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub campaign_type: CampaignType,
    pub targeting: TargetingCriteria,
    pub budget: Budget,
    pub schedule: CampaignSchedule,
    pub creative_assets: Vec<CreativeAsset>,
    pub status: CampaignStatus,
    pub performance: CampaignPerformance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "campaign_type")]
pub enum CampaignType {
    Video,
    Display,
    Native,
    InStream,
    OutStream,
    Discovery,
    Masthead,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetingCriteria {
    pub demographics: Option<Demographics>,
    pub interests: Vec<String>,
    pub keywords: Vec<String>,
    pub locations: Vec<String>,
    pub languages: Vec<String>,
    pub devices: Vec<DeviceType>,
    pub platforms: Vec<PlatformType>,
    pub time_of_day: Option<Vec<i32>>,
    pub day_of_week: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Demographics {
    pub age_range: Option<(i32, i32)>,
    pub gender: Option<Vec<String>>,
    pub income_level: Option<String>,
    pub education_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "device_type")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    ConnectedTV,
    SmartSpeaker,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "platform_type")]
pub enum PlatformType {
    Web,
    iOS,
    Android,
    SmartTV,
    Roku,
    FireTV,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Budget {
    pub total_amount: f64,
    pub daily_amount: f64,
    pub currency: String,
    pub bidding_strategy: BiddingStrategy,
    pub bid_amount: Option<f64>,
    pub cpc_max: Option<f64>,
    pub cpm_max: Option<f64>,
    pub cpv_max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "bidding_strategy")]
pub enum BiddingStrategy {
    ManualCPC,
    ManualCPM,
    ManualCPV,
    TargetCPA,
    TargetROAS,
    MaximizeConversions,
    MaximizeClicks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignSchedule {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub timezone: String,
    pub frequency_cap: Option<FrequencyCap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrequencyCap {
    pub impressions: i32,
    pub time_period: TimePeriod,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "time_period")]
pub enum TimePeriod {
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreativeAsset {
    pub id: Uuid,
    pub asset_type: CreativeType,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub duration: Option<i64>,
    pub dimensions: Option<(i32, i32)>,
    pub file_size: Option<i64>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "creative_type")]
pub enum CreativeType {
    Image,
    Video,
    HTML,
    Text,
    Audio,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "campaign_status")]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CampaignPerformance {
    pub impressions: i64,
    pub clicks: i64,
    pub conversions: i64,
    pub spend: f64,
    pub revenue: f64,
    pub ctr: f64,
    pub cpc: f64,
    pub cpm: f64,
    pub cpa: f64,
    pub roas: f64,
    pub view_through_rate: f64,
    pub completion_rate: f64,
    pub engagement_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NFT {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub video_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub metadata: serde_json::Value,
    pub contract_address: String,
    pub token_id: String,
    pub blockchain: String,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub status: NFTStatus,
    pub created_at: DateTime<Utc>,
    pub minted_at: Option<DateTime<Utc>>,
    pub sold_at: Option<DateTime<Utc>>,
    pub royalty_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "nft_status")]
pub enum NFTStatus {
    Draft,
    Minting,
    Minted,
    Listed,
    Sold,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Course {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub description: String,
    pub category: String,
    pub thumbnail_url: String,
    pub video_ids: Vec<Uuid>,
    pub pricing: CoursePricing,
    pub curriculum: Vec<CourseModule>,
    pub requirements: Vec<String>,
    pub what_you_learn: Vec<String>,
    pub target_audience: Vec<String>,
    pub status: CourseStatus,
    pub enrollment_count: i64,
    pub rating: f64,
    pub review_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoursePricing {
    pub pricing_model: PricingModel,
    pub price: Option<f64>,
    pub currency: String,
    pub free_trial_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseModule {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub video_ids: Vec<Uuid>,
    pub order: i32,
    pub duration: i64,
    pub is_published: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(type_name = "course_status")]
pub enum CourseStatus {
    Draft,
    Published,
    Unpublished,
    Archived,
}

impl Monetization {
    pub fn new(user_id: Uuid, monetization_type: MonetizationType) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            monetization_type,
            status: MonetizationStatus::Pending,
            settings: MonetizationSettings {
                ad_settings: None,
                premium_settings: None,
                membership_settings: None,
                tip_settings: None,
                brand_deal_settings: None,
                merchandise_settings: None,
                nft_settings: None,
                course_settings: None,
            },
            revenue: RevenueMetrics {
                total_revenue: 0.0,
                monthly_revenue: 0.0,
                yearly_revenue: 0.0,
                revenue_by_source: Vec::new(),
                payout_amount: 0.0,
                payout_currency: "USD".to_string(),
                last_payout_date: None,
                next_payout_date: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, MonetizationStatus::Active)
    }

    pub fn can_payout(&self) -> bool {
        self.is_active() && self.revenue.payout_amount > 0.0
    }

    pub fn get_revenue_percentage(&self) -> f64 {
        match self.monetization_type {
            MonetizationType::Ads => 0.70,
            MonetizationType::Premium => 0.80,
            MonetizationType::Membership => 0.90,
            MonetizationType::Tips => 0.90,
            MonetizationType::SuperChat => 0.90,
            MonetizationType::BrandDeals => 0.75,
            MonetizationType::Merchandise => 0.80,
            MonetizationType::NFTs => 0.95,
            MonetizationType::Courses => 0.85,
            MonetizationType::ApiAccess => 0.60,
            MonetizationType::VREvents => 0.75,
        }
    }
}

impl Payment {
    pub fn new(
        user_id: Uuid,
        payment_type: PaymentType,
        amount: f64,
        currency: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            payment_type,
            amount,
            currency,
            status: PaymentStatus::Pending,
            transaction_id: None,
            created_at: Utc::now(),
            completed_at: None,
            refunded_at: None,
            refund_amount: None,
            refund_reason: None,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, PaymentStatus::Completed)
    }

    pub fn can_refund(&self) -> bool {
        self.is_completed() && self.refunded_at.is_none()
    }
}

impl CreatorPayout {
    pub fn new(
        user_id: Uuid,
        amount: f64,
        currency: String,
        payout_method: PayoutMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            amount,
            currency,
            payout_method,
            status: PayoutStatus::Pending,
            transaction_id: None,
            created_at: Utc::now(),
            processed_at: None,
            completed_at: None,
            failed_at: None,
            failure_reason: None,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, PayoutStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, PayoutStatus::Failed)
    }
}

impl AdCampaign {
    pub fn new(
        advertiser_id: Uuid,
        name: String,
        campaign_type: CampaignType,
        budget: Budget,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            advertiser_id,
            name,
            description: None,
            campaign_type,
            targeting: TargetingCriteria {
                demographics: None,
                interests: Vec::new(),
                keywords: Vec::new(),
                locations: Vec::new(),
                languages: Vec::new(),
                devices: Vec::new(),
                platforms: Vec::new(),
                time_of_day: None,
                day_of_week: None,
            },
            budget,
            schedule: CampaignSchedule {
                start_date: Utc::now(),
                end_date: None,
                timezone: "UTC".to_string(),
                frequency_cap: None,
            },
            creative_assets: Vec::new(),
            status: CampaignStatus::Draft,
            performance: CampaignPerformance {
                impressions: 0,
                clicks: 0,
                conversions: 0,
                spend: 0.0,
                revenue: 0.0,
                ctr: 0.0,
                cpc: 0.0,
                cpm: 0.0,
                cpa: 0.0,
                roas: 0.0,
                view_through_rate: 0.0,
                completion_rate: 0.0,
                engagement_rate: 0.0,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, CampaignStatus::Active)
    }

    pub fn has_budget_remaining(&self) -> bool {
        self.performance.spend < self.budget.total_amount
    }
}