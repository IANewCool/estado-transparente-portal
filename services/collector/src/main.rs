//! Collector Service - Downloads and stores raw artifacts from public sources
//!
//! Responsibilities:
//! - Fetch resources from public URLs (CSV, JSON, HTML, PDF)
//! - Apply rate limiting to avoid degrading source sites
//! - Cache responses to avoid redundant downloads
//! - Store raw artifacts in MinIO or filesystem
//! - Register artifact metadata in database
//! - Track job runs for auditing

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "collector", about = "Collects raw artifacts from public sources")]
struct Args {
    /// Source identifier (string key)
    #[arg(long)]
    source_id: String,

    /// URL to fetch
    #[arg(long)]
    url: String,

    /// Force re-download even if cached
    #[arg(long, default_value = "false")]
    force: bool,

    /// Dry run - don't save to database
    #[arg(long, default_value = "false")]
    dry_run: bool,
}

#[derive(Debug)]
struct ArtifactMeta {
    artifact_id: Uuid,
    source_id: String,
    url: String,
    captured_at: DateTime<Utc>,
    content_hash: String,
    mime_type: String,
    size_bytes: i64,
    storage_kind: String,
    storage_path: String,
}

#[derive(Debug, Clone)]
struct Config {
    db_url: String,
    raw_store: String,
    raw_fs_dir: PathBuf,
    rate_limit_ms: u64,
}

impl Config {
    fn from_env() -> Result<Self> {
        Ok(Self {
            db_url: std::env::var("DB_URL").context("DB_URL env var missing")?,
            raw_store: std::env::var("RAW_STORE").unwrap_or_else(|_| "fs".to_string()),
            raw_fs_dir: PathBuf::from(
                std::env::var("RAW_FS_DIR").unwrap_or_else(|_| "./data/raw".to_string()),
            ),
            rate_limit_ms: std::env::var("RATE_LIMIT_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
        })
    }
}

/// Check if artifact with same hash already exists
async fn check_existing_artifact(pool: &PgPool, content_hash: &str) -> Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT artifact_id FROM artifacts WHERE content_hash = $1 LIMIT 1",
    )
    .bind(content_hash)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0))
}

/// Create a new job run record
async fn create_job_run(pool: &PgPool, source_id: &str) -> Result<Uuid> {
    let job_run_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO job_runs (job_run_id, component, source_id, status, detail)
        VALUES ($1, 'collector', $2, 'running', '{}')
        "#,
    )
    .bind(job_run_id)
    .bind(source_id)
    .execute(pool)
    .await?;

    Ok(job_run_id)
}

/// Update job run status
async fn finish_job_run(
    pool: &PgPool,
    job_run_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE job_runs
        SET finished_at = now(), status = $2, error = $3
        WHERE job_run_id = $1
        "#,
    )
    .bind(job_run_id)
    .bind(status)
    .bind(error)
    .execute(pool)
    .await?;

    Ok(())
}

/// Save artifact to filesystem
async fn save_to_fs(config: &Config, artifact_id: Uuid, bytes: &[u8]) -> Result<String> {
    let dir = &config.raw_fs_dir;
    fs::create_dir_all(dir).await?;

    let filename = format!("{}.raw", artifact_id);
    let path = dir.join(&filename);

    fs::write(&path, bytes).await?;

    Ok(path.to_string_lossy().to_string())
}

/// Insert artifact record into database
async fn insert_artifact(pool: &PgPool, meta: &ArtifactMeta) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO artifacts
        (artifact_id, source_id, url, captured_at, content_hash, mime_type, size_bytes, storage_kind, storage_path, parsed_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')
        "#,
    )
    .bind(meta.artifact_id)
    .bind(&meta.source_id)
    .bind(&meta.url)
    .bind(meta.captured_at)
    .bind(&meta.content_hash)
    .bind(&meta.mime_type)
    .bind(meta.size_bytes)
    .bind(&meta.storage_kind)
    .bind(&meta.storage_path)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let config = Config::from_env()?;

    println!("=== Estado Transparente Collector ===");
    println!("Source: {}", args.source_id);
    println!("URL: {}", args.url);
    println!("Storage: {}", config.raw_store);

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await
        .context("Failed to connect to database")?;

    // Create job run
    let job_run_id = if !args.dry_run {
        Some(create_job_run(&pool, &args.source_id).await?)
    } else {
        None
    };

    let result = async {
        // Rate limit: wait before request
        println!("Rate limit: waiting {}ms...", config.rate_limit_ms);
        sleep(Duration::from_millis(config.rate_limit_ms)).await;

        // Fetch URL
        println!("Fetching URL...");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("EstadoTransparente/1.0 (portal ciudadano independiente)")
            .build()?;

        let resp = client
            .get(&args.url)
            .send()
            .await?
            .error_for_status()
            .context("HTTP request failed")?;

        let mime = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = resp.bytes().await?;
        let size_bytes = bytes.len() as i64;

        // Calculate hash
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let content_hash = format!("sha256:{:x}", hasher.finalize());

        println!("Downloaded: {} bytes, mime: {}", size_bytes, mime);
        println!("Hash: {}", content_hash);

        // Check for existing artifact with same hash
        if !args.force {
            if let Some(existing_id) = check_existing_artifact(&pool, &content_hash).await? {
                println!("Artifact already exists with same hash: {}", existing_id);
                println!("Use --force to re-download");
                return Ok(existing_id);
            }
        }

        let artifact_id = Uuid::new_v4();
        let captured_at = Utc::now();

        // Save to storage (filesystem for MVP)
        let storage_path = save_to_fs(&config, artifact_id, &bytes).await?;
        let storage_kind = "fs".to_string();

        println!("Saved to {}: {}", storage_kind, storage_path);

        let meta = ArtifactMeta {
            artifact_id,
            source_id: args.source_id.clone(),
            url: args.url.clone(),
            captured_at,
            content_hash,
            mime_type: mime,
            size_bytes,
            storage_kind,
            storage_path,
        };

        // Insert into database
        if !args.dry_run {
            insert_artifact(&pool, &meta).await?;
            println!("Artifact registered: {}", artifact_id);
        } else {
            println!("Dry run - artifact not saved to database");
            println!("Would create artifact: {}", artifact_id);
        }

        Ok::<Uuid, anyhow::Error>(artifact_id)
    }
    .await;

    // Update job run status
    if let Some(job_id) = job_run_id {
        match &result {
            Ok(_) => finish_job_run(&pool, job_id, "ok", None).await?,
            Err(e) => finish_job_run(&pool, job_id, "failed", Some(&e.to_string())).await?,
        }
    }

    let artifact_id = result?;
    println!("\n=== Collection Complete ===");
    println!("Artifact ID: {}", artifact_id);
    println!(
        "Ready for parsing with: cargo run --bin parser -- --artifact-id {}",
        artifact_id
    );

    Ok(())
}
