use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub ai: AIConfig,
    pub s3: S3Config,
    pub security: SecurityConfig,
    pub streaming: StreamingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub clickhouse_url: String,
    pub elasticsearch_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub model_endpoint: String,
    pub api_key: String,
    pub timeout: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub public_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub bcrypt_cost: u32,
    pub max_login_attempts: u32,
    pub login_attempt_window: u64,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub hls_segment_duration: u32,
    pub dash_segment_duration: u32,
    pub max_bitrate: u32,
    pub min_bitrate: u32,
    pub transcoding_profiles: Vec<TranscodingProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodingProfile {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub codec: String,
    pub audio_bitrate: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::default();
        
        // Merge from environment variables
        settings.merge(config::Environment::with_prefix("VIDEO_PLATFORM"))?;
        
        settings.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
                workers: num_cpus::get(),
            },
            database: DatabaseConfig {
                postgres_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://postgres:password@localhost/video_platform".to_string()),
                clickhouse_url: env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string()),
                elasticsearch_url: env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| "http://localhost:9200".to_string()),
                max_connections: 100,
                min_connections: 10,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: 100,
                connection_timeout: 5000,
            },
            ai: AIConfig {
                model_endpoint: env::var("AI_MODEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:8000".to_string()),
                api_key: env::var("AI_API_KEY").unwrap_or_default(),
                timeout: 30000,
                max_retries: 3,
            },
            s3: S3Config {
                endpoint: env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
                access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
                secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
                bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "video-platform".to_string()),
                region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                public_url: env::var("S3_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            },
            security: SecurityConfig {
                jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string()),
                jwt_expires_in: "7d".to_string(),
                bcrypt_cost: 12,
                max_login_attempts: 5,
                login_attempt_window: 900, // 15 minutes
                rate_limit: RateLimitConfig {
                    requests_per_minute: 60,
                    burst_size: 10,
                },
            },
            streaming: StreamingConfig {
                hls_segment_duration: 6,
                dash_segment_duration: 4,
                max_bitrate: 8000,
                min_bitrate: 300,
                transcoding_profiles: vec![
                    TranscodingProfile {
                        name: "1080p".to_string(),
                        width: 1920,
                        height: 1080,
                        bitrate: 5000,
                        codec: "h264".to_string(),
                        audio_bitrate: 128,
                    },
                    TranscodingProfile {
                        name: "720p".to_string(),
                        width: 1280,
                        height: 720,
                        bitrate: 3000,
                        codec: "h264".to_string(),
                        audio_bitrate: 96,
                    },
                    TranscodingProfile {
                        name: "480p".to_string(),
                        width: 854,
                        height: 480,
                        bitrate: 1500,
                        codec: "h264".to_string(),
                        audio_bitrate: 64,
                    },
                    TranscodingProfile {
                        name: "360p".to_string(),
                        width: 640,
                        height: 360,
                        bitrate: 800,
                        codec: "h264".to_string(),
                        audio_bitrate: 48,
                    },
                    TranscodingProfile {
                        name: "mobile".to_string(),
                        width: 426,
                        height: 240,
                        bitrate: 400,
                        codec: "h264".to_string(),
                        audio_bitrate: 32,
                    },
                ],
            },
        }
    }
}