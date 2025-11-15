use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use redis::Client;
use clickhouse::Client as ClickHouseClient;
use elasticsearch::Elasticsearch;
use std::sync::Arc;
use crate::config::Config;

pub type DbPool = Pool<Postgres>;
pub type RedisClient = Client;
pub type ClickHouse = ClickHouseClient;
pub type ElasticSearch = Elasticsearch;

#[derive(Clone)]
pub struct DatabaseConnections {
    pub postgres: DbPool,
    pub redis: RedisClient,
    pub clickhouse: ClickHouse,
    pub elasticsearch: ElasticSearch,
}

impl DatabaseConnections {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize PostgreSQL connection pool
        let postgres = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .connect(&config.database.postgres_url)
            .await?;

        // Initialize Redis client
        let redis = Client::open(config.redis.url.clone())?;

        // Initialize ClickHouse client
        let clickhouse = ClickHouseClient::default()
            .with_url(&config.database.clickhouse_url)
            .with_user("default")
            .with_password("")
            .build()?;

        // Initialize Elasticsearch client
        let elasticsearch = Elasticsearch::default()
            .with_url(&config.database.elasticsearch_url)
            .build()?;

        Ok(Self {
            postgres,
            redis,
            clickhouse,
            elasticsearch,
        })
    }

    pub fn get_postgres(&self) -> &DbPool {
        &self.postgres
    }

    pub fn get_redis(&self) -> &RedisClient {
        &self.redis
    }

    pub fn get_clickhouse(&self) -> &ClickHouse {
        &self.clickhouse
    }

    pub fn get_elasticsearch(&self) -> &ElasticSearch {
        &self.elasticsearch
    }
}

// Database initialization functions
pub async fn init_db(config: &Config) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.postgres_url)
        .await?;

    // Run database migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn init_redis(config: &Config) -> Result<RedisClient, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::open(config.redis.url.clone())?;
    Ok(client)
}

// Database schema migrations
async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create users table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) UNIQUE NOT NULL,
            username VARCHAR(50) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            full_name VARCHAR(100),
            avatar_url VARCHAR(500),
            bio TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            last_login TIMESTAMP WITH TIME ZONE,
            is_active BOOLEAN DEFAULT TRUE,
            is_verified BOOLEAN DEFAULT FALSE,
            is_premium BOOLEAN DEFAULT FALSE,
            trust_level INTEGER DEFAULT 1 CHECK (trust_level >= 1 AND trust_level <= 5),
            role VARCHAR(20) DEFAULT 'viewer' CHECK (role IN ('viewer', 'creator', 'brand', 'advertiser', 'moderator', 'admin')),
            settings JSONB DEFAULT '{}',
            preferences JSONB DEFAULT '{}'
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create videos table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS videos (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            thumbnail_url VARCHAR(500),
            video_url VARCHAR(500) NOT NULL,
            duration INTEGER NOT NULL,
            size BIGINT NOT NULL,
            width INTEGER,
            height INTEGER,
            fps INTEGER,
            bitrate INTEGER,
            codec VARCHAR(10),
            audio_codec VARCHAR(10),
            audio_bitrate INTEGER,
            emotion_index FLOAT,
            topic_category VARCHAR(50),
            engagement_prediction FLOAT,
            ranking_score FLOAT DEFAULT 0.0,
            view_count BIGINT DEFAULT 0,
            like_count BIGINT DEFAULT 0,
            comment_count BIGINT DEFAULT 0,
            share_count BIGINT DEFAULT 0,
            save_count BIGINT DEFAULT 0,
            status VARCHAR(20) DEFAULT 'processing' CHECK (status IN ('processing', 'ready', 'failed', 'private', 'unlisted')),
            visibility VARCHAR(20) DEFAULT 'public' CHECK (visibility IN ('public', 'private', 'unlisted')),
            tags TEXT[],
            metadata JSONB DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            published_at TIMESTAMP WITH TIME ZONE,
            processing_completed_at TIMESTAMP WITH TIME ZONE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create comments table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            video_id UUID NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            toxicity_score FLOAT DEFAULT 0.0,
            like_count INTEGER DEFAULT 0,
            reply_count INTEGER DEFAULT 0,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            is_deleted BOOLEAN DEFAULT FALSE,
            is_pinned BOOLEAN DEFAULT FALSE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create playlists table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS playlists (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            thumbnail_url VARCHAR(500),
            visibility VARCHAR(20) DEFAULT 'public' CHECK (visibility IN ('public', 'private', 'unlisted')),
            video_order JSONB DEFAULT '[]',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create playlist_videos junction table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS playlist_videos (
            playlist_id UUID NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            video_id UUID NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
            added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            position INTEGER NOT NULL,
            PRIMARY KEY (playlist_id, video_id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create likes table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS likes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            video_id UUID NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            UNIQUE(user_id, video_id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create video_views table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS video_views (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            video_id UUID NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
            user_id UUID REFERENCES users(id) ON DELETE SET NULL,
            ip_address INET,
            user_agent TEXT,
            watch_duration INTEGER DEFAULT 0,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            UNIQUE(video_id, user_id, created_at)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create monetization tables
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS creator_revenue (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            video_id UUID REFERENCES videos(id) ON DELETE CASCADE,
            revenue_type VARCHAR(20) NOT NULL CHECK (revenue_type IN ('ads', 'premium', 'membership', 'tips', 'superchat', 'brand_deal')),
            amount DECIMAL(10, 2) NOT NULL,
            currency VARCHAR(3) DEFAULT 'USD',
            status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'cancelled')),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            paid_at TIMESTAMP WITH TIME ZONE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create moderation queue table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS moderation_queue (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            content_type VARCHAR(20) NOT NULL CHECK (content_type IN ('video', 'comment', 'user')),
            content_id UUID NOT NULL,
            reported_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            reason TEXT,
            priority INTEGER DEFAULT 1 CHECK (priority >= 1 AND priority <= 5),
            status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'reviewed', 'approved', 'rejected')),
            moderator_id UUID REFERENCES users(id) ON DELETE SET NULL,
            decision TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            reviewed_at TIMESTAMP WITH TIME ZONE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes for better performance
    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_videos_user_id ON videos(user_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_videos_status ON videos(status)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_videos_created_at ON videos(created_at)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_comments_video_id ON comments(video_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_comments_user_id ON comments(user_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_video_views_video_id ON video_views(video_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "CREATE INDEX IF NOT EXISTS idx_video_views_user_id ON video_views(user_id)"
    )
    .execute(pool)
    .await?;

    Ok(())
}