use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{auth, database, utils};
use crate::database::DbPool;
use crate::config::Config;
use sqlx::FromRow;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamManifestRequest {
    pub video_id: Uuid,
    pub format: String, // "hls", "dash", "webrtc"
    pub quality: Option<String>, // "1080p", "720p", "480p", "360p"
    pub adaptive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamManifest {
    pub format: String,
    pub qualities: Vec<QualityLevel>,
    pub adaptive_streams: Option<AdaptiveStreams>,
    pub drm_protection: Option<DrmProtection>,
    pub metadata: StreamMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityLevel {
    pub quality: String,
    pub bitrate: i32,
    pub width: i32,
    pub height: i32,
    pub codecs: Vec<String>,
    pub manifest_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdaptiveStreams {
    pub video_streams: Vec<QualityLevel>,
    pub audio_streams: Vec<AudioStream>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioStream {
    pub language: String,
    pub bitrate: i32,
    pub channels: i32,
    pub sample_rate: i32,
    pub manifest_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrmProtection {
    pub widevine: Option<DrmKey>,
    pub playready: Option<DrmKey>,
    pub fairplay: Option<DrmKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrmKey {
    pub key_id: String,
    pub key_server_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub duration: f64,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub creator: String,
    pub tags: Vec<String>,
}

// Get stream manifest
pub async fn get_stream_manifest(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    req: web::Json<StreamManifestRequest>,
) -> Result<HttpResponse> {
    // Get video information
    let video = sqlx::query!(
        "SELECT id, title, description, thumbnail_url, duration, video_url, user_id FROM videos WHERE id = $1",
        req.video_id
    )
    .fetch_optional(pool.as_ref())
    .await?;

    let video = match video {
        Some(video) => video,
        None => {
            return Ok(HttpResponse::NotFound().json(utils::ErrorResponse {
                success: false,
                message: "Video not found".to_string(),
                errors: None,
            }));
        }
    };

    // Check if video is ready for streaming
    if video.video_url.is_empty() {
        return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
            success: false,
            message: "Video is not ready for streaming".to_string(),
            errors: None,
        }));
    }

    // Generate manifest based on format
    let manifest = match req.format.as_str() {
        "hls" => generate_hls_manifest(&video, &config, req.quality.as_deref(), req.adaptive.unwrap_or(false)).await,
        "dash" => generate_dash_manifest(&video, &config, req.quality.as_deref(), req.adaptive.unwrap_or(false)).await,
        "webrtc" => generate_webrtc_manifest(&video, &config).await,
        _ => {
            return Ok(HttpResponse::BadRequest().json(utils::ErrorResponse {
                success: false,
                message: "Unsupported streaming format".to_string(),
                errors: None,
            }));
        }
    };

    match manifest {
        Ok(manifest_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Stream manifest generated".to_string(),
                data: Some(serde_json::to_value(manifest_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to generate manifest: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to generate stream manifest".to_string(),
                errors: None,
            }))
        }
    }
}

// Generate HLS manifest
async fn generate_hls_manifest(
    video: &sqlx::postgres::PgRow,
    config: &Config,
    quality: Option<&str>,
    adaptive: bool,
) -> Result<StreamManifest, Box<dyn std::error::Error + Send + Sync>> {
    let video_id = video.get::<Uuid, _>("id");
    let duration = video.get::<i32, _>("duration") as f64;
    let title = video.get::<String, _>("title");
    let description = video.get::<Option<String>, _>("description");
    let thumbnail_url = video.get::<Option<String>, _>("thumbnail_url");

    // Get creator username
    let creator_username = sqlx::query!(
        "SELECT username FROM users WHERE id = $1",
        video.get::<Uuid, _>("user_id")
    )
    .fetch_one(&database::init_db(config).await.unwrap())
    .await?
    .username;

    if adaptive {
        // Generate adaptive HLS streams
        let video_streams = vec
![
            QualityLevel {
                quality: "1080p".to_string(),
                bitrate: 5000,
                width: 1920,
                height: 1080,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/hls/{}/1080p.m3u8", config.streaming.hls_segment_duration, video_id),
            },
            QualityLevel {
                quality: "720p".to_string(),
                bitrate: 3000,
                width: 1280,
                height: 720,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/hls/{}/720p.m3u8", config.streaming.hls_segment_duration, video_id),
            },
            QualityLevel {
                quality: "480p".to_string(),
                bitrate: 1500,
                width: 854,
                height: 480,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/hls/{}/480p.m3u8", config.streaming.hls_segment_duration, video_id),
            },
            QualityLevel {
                quality: "360p".to_string(),
                bitrate: 800,
                width: 640,
                height: 360,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/hls/{}/360p.m3u8", config.streaming.hls_segment_duration, video_id),
            },
        ];

        let audio_streams = vec
![
            AudioStream {
                language: "en".to_string(),
                bitrate: 128,
                channels: 2,
                sample_rate: 44100,
                manifest_url: format!("{}/hls/{}/audio_en.m3u8", config.streaming.hls_segment_duration, video_id),
            },
        ];

        Ok(StreamManifest {
            format: "hls".to_string(),
            qualities: video_streams.clone(),
            adaptive_streams: Some(AdaptiveStreams {
                video_streams,
                audio_streams,
            }),
            drm_protection: None,
            metadata: StreamMetadata {
                duration,
                title,
                description,
                thumbnail_url,
                creator: creator_username,
                tags: vec
![], // Placeholder
            },
        })
    } else {
        // Generate single quality HLS stream
        let selected_quality = match quality {
            Some("1080p") => "1080p",
            Some("720p") => "720p",
            Some("480p") => "480p",
            Some("360p") => "360p",
            _ => "720p", // Default
        };

        let bitrate = match selected_quality {
            "1080p" => 5000,
            "720p" => 3000,
            "480p" => 1500,
            "360p" => 800,
            _ => 3000,
        };

        let width = match selected_quality {
            "1080p" => 1920,
            "720p" => 1280,
            "480p" => 854,
            "360p" => 640,
            _ => 1280,
        };

        let height = match selected_quality {
            "1080p" => 1080,
            "720p" => 720,
            "480p" => 480,
            "360p" => 360,
            _ => 720,
        };

        Ok(StreamManifest {
            format: "hls".to_string(),
            qualities: vec
![QualityLevel {
                quality: selected_quality.to_string(),
                bitrate,
                width,
                height,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/hls/{}/{}.m3u8", config.streaming.hls_segment_duration, video_id, selected_quality),
            }],
            adaptive_streams: None,
            drm_protection: None,
            metadata: StreamMetadata {
                duration,
                title,
                description,
                thumbnail_url,
                creator: creator_username,
                tags: vec
![], // Placeholder
            },
        })
    }
}

// Generate DASH manifest
async fn generate_dash_manifest(
    video: &sqlx::postgres::PgRow,
    config: &Config,
    quality: Option<&str>,
    adaptive: bool,
) -> Result<StreamManifest, Box<dyn std::error::Error + Send + Sync>> {
    let video_id = video.get::<Uuid, _>("id");
    let duration = video.get::<i32, _>("duration") as f64;
    let title = video.get::<String, _>("title");
    let description = video.get::<Option<String>, _>("description");
    let thumbnail_url = video.get::<Option<String>, _>("thumbnail_url");

    // Get creator username
    let creator_username = sqlx::query!(
        "SELECT username FROM users WHERE id = $1",
        video.get::<Uuid, _>("user_id")
    )
    .fetch_one(&database::init_db(config).await.unwrap())
    .await?
    .username;

    if adaptive {
        // Generate adaptive DASH streams
        let video_streams = vec
![
            QualityLevel {
                quality: "1080p".to_string(),
                bitrate: 5000,
                width: 1920,
                height: 1080,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/dash/{}/1080p.mpd", config.streaming.dash_segment_duration, video_id),
            },
            QualityLevel {
                quality: "720p".to_string(),
                bitrate: 3000,
                width: 1280,
                height: 720,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/dash/{}/720p.mpd", config.streaming.dash_segment_duration, video_id),
            },
            QualityLevel {
                quality: "480p".to_string(),
                bitrate: 1500,
                width: 854,
                height: 480,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/dash/{}/480p.mpd", config.streaming.dash_segment_duration, video_id),
            },
            QualityLevel {
                quality: "360p".to_string(),
                bitrate: 800,
                width: 640,
                height: 360,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/dash/{}/360p.mpd", config.streaming.dash_segment_duration, video_id),
            },
        ];

        let audio_streams = vec
![
            AudioStream {
                language: "en".to_string(),
                bitrate: 128,
                channels: 2,
                sample_rate: 44100,
                manifest_url: format!("{}/dash/{}/audio_en.mpd", config.streaming.dash_segment_duration, video_id),
            },
        ];

        Ok(StreamManifest {
            format: "dash".to_string(),
            qualities: video_streams.clone(),
            adaptive_streams: Some(AdaptiveStreams {
                video_streams,
                audio_streams,
            }),
            drm_protection: Some(DrmProtection {
                widevine: Some(DrmKey {
                    key_id: "widevine_key_id".to_string(),
                    key_server_url: "https://drm.example.com/widevine".to_string(),
                }),
                playready: Some(DrmKey {
                    key_id: "playready_key_id".to_string(),
                    key_server_url: "https://drm.example.com/playready".to_string(),
                }),
                fairplay: Some(DrmKey {
                    key_id: "fairplay_key_id".to_string(),
                    key_server_url: "https://drm.example.com/fairplay".to_string(),
                }),
            }),
            metadata: StreamMetadata {
                duration,
                title,
                description,
                thumbnail_url,
                creator: creator_username,
                tags: vec
![], // Placeholder
            },
        })
    } else {
        // Generate single quality DASH stream
        let selected_quality = match quality {
            Some("1080p") => "1080p",
            Some("720p") => "720p",
            Some("480p") => "480p",
            Some("360p") => "360p",
            _ => "720p", // Default
        };

        let bitrate = match selected_quality {
            "1080p" => 5000,
            "720p" => 3000,
            "480p" => 1500,
            "360p" => 800,
            _ => 3000,
        };

        let width = match selected_quality {
            "1080p" => 1920,
            "720p" => 1280,
            "480p" => 854,
            "360p" => 640,
            _ => 1280,
        };

        let height = match selected_quality {
            "1080p" => 1080,
            "720p" => 720,
            "480p" => 480,
            "360p" => 360,
            _ => 720,
        };

        Ok(StreamManifest {
            format: "dash".to_string(),
            qualities: vec
![QualityLevel {
                quality: selected_quality.to_string(),
                bitrate,
                width,
                height,
                codecs: vec
!["h264".to_string()],
                manifest_url: format!("{}/dash/{}/{}.mpd", config.streaming.dash_segment_duration, video_id, selected_quality),
            }],
            adaptive_streams: None,
            drm_protection: None,
            metadata: StreamMetadata {
                duration,
                title,
                description,
                thumbnail_url,
                creator: creator_username,
                tags: vec
![], // Placeholder
            },
        })
    }
}

// Generate WebRTC manifest
async fn generate_webrtc_manifest(
    video: &sqlx::postgres::PgRow,
    config: &Config,
) -> Result<StreamManifest, Box<dyn std::error::Error + Send + Sync>> {
    let video_id = video.get::<Uuid, _>("id");
    let duration = video.get::<i32, _>("duration") as f64;
    let title = video.get::<String, _>("title");
    let description = video.get::<Option<String>, _>("description");
    let thumbnail_url = video.get::<Option<String>, _>("thumbnail_url");

    // Get creator username
    let creator_username = sqlx::query!(
        "SELECT username FROM users WHERE id = $1",
        video.get::<Uuid, _>("user_id")
    )
    .fetch_one(&database::init_db(config).await.unwrap())
    .await?
    .username;

    // WebRTC typically uses a single quality for low latency
    Ok(StreamManifest {
        format: "webrtc".to_string(),
        qualities: vec
![QualityLevel {
            quality: "live".to_string(),
            bitrate: 3000,
            width: 1280,
            height: 720,
            codecs: vec
!["h264".to_string()],
            manifest_url: format!("{}/webrtc/{}.json", config.streaming.hls_segment_duration, video_id),
        }],
        adaptive_streams: None,
        drm_protection: None,
        metadata: StreamMetadata {
            duration,
            title,
            description,
            thumbnail_url,
            creator: creator_username,
            tags: vec
![], // Placeholder
        },
    })
}

// Process video for streaming
pub async fn process_video_streaming(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    video_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get video information
    let video = sqlx::query!(
        "SELECT video_url, duration, width, height, fps, bitrate, codec, audio_codec, audio_bitrate FROM videos WHERE id = $1",
        video_id
    )
    .fetch_one(pool.as_ref())
    .await?;

    let video_url = video.video_url;
    let duration = video.duration;
    let width = video.width;
    let height = video.height;
    let fps = video.fps;
    let bitrate = video.bitrate;
    let codec = video.codec;
    let audio_codec = video.audio_codec;
    let audio_bitrate = video.audio_bitrate;

    // Generate streaming formats
    generate_streaming_formats(&video_url, &video_id, duration, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, &config).await?;

    // Update video with streaming information
    sqlx::query!(
        "UPDATE videos SET streaming_ready = TRUE WHERE id = $1",
        video_id
    )
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

// Generate streaming formats
async fn generate_streaming_formats(
    video_url: &str,
    video_id: &Uuid,
    duration: i32,
    width: Option<i32>,
    height: Option<i32>,
    fps: Option<i32>,
    bitrate: Option<i32>,
    codec: Option<String>,
    audio_codec: Option<String>,
    audio_bitrate: Option<i32>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create streaming directory
    let streaming_dir = format!("./storage/streaming/{}", video_id);
    fs::create_dir_all(&streaming_dir)?;

    // Generate HLS streams
    generate_hls_streams(video_url, video_id, duration, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, config).await?;

    // Generate DASH streams
    generate_dash_streams(video_url, video_id, duration, width, height, fps, bitrate, codec, audio_codec, audio_bitrate, config).await?;

    // Generate WebRTC stream
    generate_webrtc_stream(video_url, video_id, config).await?;

    Ok(())
}

// Generate HLS streams
async fn generate_hls_streams(
    video_url: &str,
    video_id: &Uuid,
    duration: i32,
    width: Option<i32>,
    height: Option<i32>,
    fps: Option<i32>,
    bitrate: Option<i32>,
    codec: Option<String>,
    audio_codec: Option<String>,
    audio_bitrate: Option<i32>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // HLS segment duration
    let segment_duration = config.streaming.hls_segment_duration;

    // Generate master playlist
    let master_playlist = generate_hls_master_playlist(video_id, segment_duration).await?;

    // Write master playlist
    let master_path = format!("./storage/streaming/{}/master.m3u8", video_id);
    fs::write(&master_path, master_playlist)?;

    // Generate quality-specific playlists and segments
    let qualities = vec
![
        (1080, 5000),  // 1080p
        (720, 3000),   // 720p
        (480, 1500),   // 480p
        (360, 800),    // 360p
    ];

    for (quality, bitrate) in qualities {
        // Generate playlist for this quality
        let playlist = generate_hls_playlist(video_id, quality, bitrate, segment_duration, duration).await?;
        
        // Write playlist
        let playlist_path = format!("./storage/streaming/{}/{}p.m3u8", video_id, quality);
        fs::write(&playlist_path, playlist)?;

        // In production, this would use FFmpeg to transcode the video
        // For now, create placeholder segment files
        generate_hls_segments(video_url, video_id, quality, bitrate, segment_duration).await?;
    }

    Ok(())
}

// Generate HLS master playlist
async fn generate_hls_master_playlist(video_id: &Uuid, segment_duration: u32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut playlist = String::new();
    
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:6\n");
    playlist.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=500000,RESOLUTION=1920x1080,CODECS=\"avc1.640028,mp4a.40.2\"\n");
    playlist.push_str("1080p.m3u8\n");
    playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=3000000,RESOLUTION=1280x720,CODECS=\"avc1.640028,mp4a.40.2\"\n");
    playlist.push_str("720p.m3u8\n");
    playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=1500000,RESOLUTION=854x480,CODECS=\"avc1.640028,mp4a.40.2\"\n");
    playlist.push_str("480p.m3u8\n");
    playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360,CODECS=\"avc1.640028,mp4a.40.2\"\n");
    playlist.push_str("360p.m3u8\n");

    Ok(playlist)
}

// Generate HLS playlist for specific quality
async fn generate_hls_playlist(video_id: &Uuid, quality: i32, bitrate: i32, segment_duration: u32, duration: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut playlist = String::new();
    
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:6\n");
    playlist.push_str("#EXT-X-TARGETDURATION:10\n");
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXTINF:6.0,\n");
    playlist.push_str("segment_0.ts\n");
    playlist.push_str("#EXTINF:6.0,\n");
    playlist.push_str("segment_1.ts\n");
    // Add more segments based on duration
    let segment_count = (duration as f64 / segment_duration as f64).ceil() as i32;
    for i in 2..segment_count {
        playlist.push_str(&format!("#EXTINF:{},\n", segment_duration));
        playlist.push_str(&format!("segment_{}.ts\n", i));
    }
    playlist.push_str("#EXT-X-ENDLIST\n");

    Ok(playlist)
}

// Generate HLS segments
async fn generate_hls_segments(video_url: &str, video_id: &Uuid, quality: i32, bitrate: i32, segment_duration: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create segments directory
    let segments_dir = format!("./storage/streaming/{}/segments", video_id);
    fs::create_dir_all(&segments_dir)?;

    // In production, this would use FFmpeg to split the video into segments
    // For now, create placeholder segment files
    let segment_count = 10; // Placeholder
    for i in 0..segment_count {
        let segment_path = format!("{}/segment_{}.ts", segments_dir, i);
        let mut file = fs::File::create(&segment_path)?;
        file.write_all(b"")?; // Empty placeholder file
    }

    Ok(())
}

// Generate DASH streams
async fn generate_dash_streams(
    video_url: &str,
    video_id: &Uuid,
    duration: i32,
    width: Option<i32>,
    height: Option<i32>,
    fps: Option<i32>,
    bitrate: Option<i32>,
    codec: Option<String>,
    audio_codec: Option<String>,
    audio_bitrate: Option<i32>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate DASH manifest
    let dash_manifest = generate_dash_manifest_file(video_id, duration).await?;

    // Write manifest
    let manifest_path = format!("./storage/streaming/{}/dash.mpd", video_id);
    fs::write(&manifest_path, dash_manifest)?;

    // In production, this would use FFmpeg to generate DASH segments
    // For now, create placeholder files
    generate_dash_segments(video_url, video_id, config).await?;

    Ok(())
}

// Generate DASH manifest file
async fn generate_dash_manifest_file(video_id: &Uuid, duration: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut manifest = String::new();
    
    manifest.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    manifest.push_str("<MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\" xmlns:cenc=\"urn:mpeg:cenc:2013\" minBufferTime=\"PT1.5S\" mediaPresentationDuration=\"PT");
    manifest.push_str(&duration.to_string());
    manifest.push_str("S\" type=\"static\" profiles=\"urn:mpeg:dash:profile:isoff-live:2011\">\n");
    manifest.push_str("  <Period duration=\"PT");
    manifest.push_str(&duration.to_string());
    manifest.push_str("S\">\n");
    manifest.push_str("    <AdaptationSet id=\"1\" contentType=\"video\" segmentAlignment=\"true\" maxWidth=\"1920\" maxHeight=\"1080\" par=\"16:9\">\n");
    manifest.push_str("      <Representation id=\"1\" mimeType=\"video/mp4\" width=\"1920\" height=\"1080\" codecs=\"avc1.640028\" frameRate=\"30\" bandwidth=\"5000000\">\n");
    manifest.push_str("        <BaseURL>1080p/init.mp4</BaseURL>\n");
    manifest.push_str("        <SegmentBase indexRange=\"0-1000\">\n");
    manifest.push_str("          <Initialization range=\"0-1000\"/>\n");
    manifest.push_str("        </SegmentBase>\n");
    manifest.push_str("      </Representation>\n");
    manifest.push_str("      <Representation id=\"2\" mimeType=\"video/mp4\" width=\"1280\" height=\"720\" codecs=\"avc1.640028\" frameRate=\"30\" bandwidth=\"3000000\">\n");
    manifest.push_str("        <BaseURL>720p/init.mp4</BaseURL>\n");
    manifest.push_str("        <SegmentBase indexRange=\"0-1000\">\n");
    manifest.push_str("          <Initialization range=\"0-1000\"/>\n");
    manifest.push_str("        </SegmentBase>\n");
    manifest.push_str("      </Representation>\n");
    manifest.push_str("      <Representation id=\"3\" mimeType=\"video/mp4\" width=\"854\" height=\"480\" codecs=\"avc1.640028\" frameRate=\"30\" bandwidth=\"1500000\">\n");
    manifest.push_str("        <BaseURL>480p/init.mp4</BaseURL>\n");
    manifest.push_str("        <SegmentBase indexRange=\"0-1000\">\n");
    manifest.push_str("          <Initialization range=\"0-1000\"/>\n");
    manifest.push_str("        </SegmentBase>\n");
    manifest.push_str("      </Representation>\n");
    manifest.push_str("      <Representation id=\"4\" mimeType=\"video/mp4\" width=\"640\" height=\"360\" codecs=\"avc1.640028\" frameRate=\"30\" bandwidth=\"800000\">\n");
    manifest.push_str("        <BaseURL>360p/init.mp4</BaseURL>\n");
    manifest.push_str("        <SegmentBase indexRange=\"0-1000\">\n");
    manifest.push_str("          <Initialization range=\"0-1000\"/>\n");
    manifest.push_str("        </SegmentBase>\n");
    manifest.push_str("      </Representation>\n");
    manifest.push_str("    </AdaptationSet>\n");
    manifest.push_str("    <AdaptationSet id=\"2\" contentType=\"audio\" lang=\"en\" segmentAlignment=\"true\">\n");
    manifest.push_str("      <Representation id=\"5\" mimeType=\"audio/mp4\" codecs=\"mp4a.40.2\" bandwidth=\"128000\">\n");
    manifest.push_str("        <BaseURL>audio_en/init.mp4</BaseURL>\n");
    manifest.push_str("        <SegmentBase indexRange=\"0-1000\">\n");
    manifest.push_str("          <Initialization range=\"0-1000\"/>\n");
    manifest.push_str("        </SegmentBase>\n");
    manifest.push_str("      </Representation>\n");
    manifest.push_str("    </AdaptationSet>\n");
    manifest.push_str("  </Period>\n");
    manifest.push_str("</MPD>\n");

    Ok(manifest)
}

// Generate DASH segments
async fn generate_dash_segments(video_url: &str, video_id: &Uuid, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create DASH directory
    let dash_dir = format!("./storage/streaming/{}/dash", video_id);
    fs::create_dir_all(&dash_dir)?;

    // In production, this would use FFmpeg to generate DASH segments
    // For now, create placeholder files
    let qualities = vec
!["1080p", "720p", "480p", "360p"];
    for quality in qualities {
        let quality_dir = format!("{}/{}", dash_dir, quality);
        fs::create_dir_all(&quality_dir)?;
        
        // Create init file
        let init_path = format!("{}/init.mp4", quality_dir);
        let mut file = fs::File::create(&init_path)?;
        file.write_all(b"")?; // Empty placeholder file

        // Create segment files
        let segment_count = 10; // Placeholder
        for i in 0..segment_count {
            let segment_path = format!("{}/segment_{}.m4s", quality_dir, i);
            let mut file = fs::File::create(&segment_path)?;
            file.write_all(b"")?; // Empty placeholder file
        }
    }

    // Create audio directory and files
    let audio_dir = format!("{}/audio_en", dash_dir);
    fs::create_dir_all(&audio_dir)?;
    
    let init_path = format!("{}/init.mp4", audio_dir);
    let mut file = fs::File::create(&init_path)?;
    file.write_all(b"")?; // Empty placeholder file

    let segment_count = 10; // Placeholder
    for i in 0..segment_count {
        let segment_path = format!("{}/segment_{}.m4s", audio_dir, i);
        let mut file = fs::File::create(&segment_path)?;
        file.write_all(b"")?; // Empty placeholder file
    }

    Ok(())
}

// Generate WebRTC stream
async fn generate_webrtc_stream(video_url: &str, video_id: &Uuid, config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create WebRTC directory
    let webrtc_dir = format!("./storage/streaming/{}/webrtc", video_id);
    fs::create_dir_all(&webrtc_dir)?;

    // Generate WebRTC configuration
    let webrtc_config = serde_json::json!({
        "video": {
            "codec": "H264",
            "profile": "baseline",
            "level": "3.0",
            "width": 1280,
            "height": 720,
            "frameRate": 30,
            "bitrate": 3000
        },
        "audio": {
            "codec": "opus",
            "sampleRate": 48000,
            "channels": 2,
            "bitrate": 128
        },
        "connection": {
            "iceServers": [
                {
                    "urls": "stun:stun.l.google.com:19302"
                }
            ],
            "iceCandidatePoolSize": 10
        }
    });

    // Write configuration file
    let config_path = format!("{}/config.json", webrtc_dir);
    fs::write(&config_path, serde_json::to_string_pretty(&webrtc_config)?)?;

    // In production, this would set up WebRTC peer connections
    // For now, create placeholder files
    let stream_path = format!("{}/stream.sdp", webrtc_dir);
    let mut file = fs::File::create(&stream_path)?;
    file.write_all(b"v=0\no=- 0 0 IN IP4 127.0.0.1\ns=WebRTC Stream\nc=IN IP4 0.0.0.0\nt=0 0\nm=video 9 UDP/TLS/RTP/SAVPF 96\nc=IN IP4 0.0.0.0\na=rtcp:9 IN IP4 0.0.0.0\na=ice-ufrag:\na=ice-pwd:\na=ice-options:trickle\na=fingerprint:sha-256 \na=setup:active\na=mid:video\na=sendonly\na=rtpmap:96 H264/90000\na=rtcp-fb:96 ccm fir\na=rtcp-fb:96 nack\na=rtcp-fb:96 nack pli\na=rtcp-fb:96 goog-remb\n")?;

    Ok(())
}

// Get streaming statistics
pub async fn get_streaming_stats(
    pool: web::Data<DbPool>,
    user: web::Data<crate::auth::User>,
) -> Result<HttpResponse> {
    // Check if user has admin or creator permissions
    if user.role != "admin" && user.role != "creator" {
        return Ok(HttpResponse::Forbidden().json(utils::ErrorResponse {
            success: false,
            message: "Insufficient permissions".to_string(),
            errors: None,
        }));
    }

    let stats = get_streaming_statistics(pool).await;

    match stats {
        Ok(stats_data) => {
            Ok(HttpResponse::Ok().json(utils::ApiResponse {
                success: true,
                message: "Streaming statistics retrieved".to_string(),
                data: Some(serde_json::to_value(stats_data).unwrap()),
            }))
        }
        Err(e) => {
            eprintln!("Failed to get streaming stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(utils::ErrorResponse {
                success: false,
                message: "Failed to get streaming statistics".to_string(),
                errors: None,
            }))
        }
    }
}

// Get streaming statistics
async fn get_streaming_statistics(
    pool: &DbPool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    // Get active streams count
    let active_streams = sqlx::query!(
        "SELECT COUNT(*) as count FROM video_streams WHERE status = 'active'"
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);

    // Get total bandwidth usage
    let bandwidth_usage = sqlx::query!(
        "SELECT COALESCE(SUM(bytes_sent), 0) as total FROM stream_stats"
    )
    .fetch_one(pool)
    .await?
    .total
    .unwrap_or(0);

    // Get average bitrate
    let avg_bitrate = sqlx::query!(
        "SELECT AVG(bitrate) as avg FROM stream_stats WHERE bitrate IS NOT NULL"
    )
    .fetch_one(pool)
    .await?
    .avg
    .unwrap_or(0.0);

    // Get streaming quality distribution
    let quality_distribution = sqlx::query!(
        r#"
        SELECT 
            quality,
            COUNT(*) as count
        FROM stream_stats
        GROUP BY quality
        ORDER BY count DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut quality_map = HashMap::new();
    for result in quality_distribution {
        quality_map.insert(result.quality, result.count.unwrap_or(0));
    }

    Ok(serde_json::json!({
        "active_streams": active_streams,
        "bandwidth_usage_gb": bandwidth_usage as f64 / (1024.0 * 1024.0 * 1024.0),
        "avg_bitrate_kbps": avg_bitrate,
        "quality_distribution": quality_map,
        "cdn_status": "healthy", // Placeholder
        "latency_stats": {
            "avg": 150, // ms
            "p95": 300,
            "p99": 500
        }
    }))
}