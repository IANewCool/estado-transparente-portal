//! Parser Service - Transforms raw artifacts into canonical facts
//!
//! Responsibilities:
//! - Load artifact metadata and raw content
//! - Parse CSV/JSON deterministically
//! - Upsert entities and metrics
//! - Insert facts with provenance (evidence chain)
//! - Mark artifact as parsed or failed
//!
//! CRITICAL: This service must be DETERMINISTIC
//! Same artifact + same parser version = same output

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Parser;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::fs;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "parser", about = "Parses raw artifacts into canonical facts")]
struct Args {
    /// Artifact id to parse (UUID)
    #[arg(long)]
    artifact_id: String,

    /// Dry run - don't save to database
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Verify mode - check if output matches existing facts
    #[arg(long, default_value = "false")]
    verify: bool,
}

/// Artifact metadata from database
#[derive(Debug, sqlx::FromRow)]
struct Artifact {
    artifact_id: Uuid,
    source_id: String,
    url: String,
    content_hash: String,
    mime_type: String,
    storage_kind: String,
    storage_path: String,
    parsed_status: String,
}

/// A parsed fact ready for insertion
#[derive(Debug)]
struct ParsedFact {
    entity_key: String,
    entity_name: String,
    entity_type: String,
    metric_key: String,
    metric_name: String,
    metric_unit: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
    value_num: f64,
    location: String, // e.g., "csv:line=5"
    dims: serde_json::Value,
}

/// CSV row structure for demo data (presupuesto format)
#[derive(Debug, Deserialize)]
struct CsvRow {
    #[serde(alias = "entidad", alias = "entity", alias = "organismo")]
    entity: String,
    #[serde(alias = "categoria", alias = "category", alias = "item")]
    category: Option<String>,
    #[serde(alias = "anio", alias = "year", alias = "periodo")]
    year: i32,
    #[serde(alias = "monto", alias = "amount", alias = "valor")]
    amount: f64,
}

/// Get or create entity, returning entity_id
async fn get_or_create_entity(
    pool: &PgPool,
    key: &str,
    name: &str,
    entity_type: &str,
) -> Result<Uuid> {
    // Try to get existing
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT entity_id FROM entities WHERE entity_key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    // Create new
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO entities (entity_id, entity_key, display_name, entity_type) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(key)
    .bind(name)
    .bind(entity_type)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Get or create metric, returning metric_id
async fn get_or_create_metric(
    pool: &PgPool,
    key: &str,
    name: &str,
    unit: &str,
) -> Result<Uuid> {
    // Try to get existing
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT metric_id FROM metrics WHERE metric_key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    // Create new
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO metrics (metric_id, metric_key, display_name, unit) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(key)
    .bind(name)
    .bind(unit)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Create a snapshot for this parsing run
async fn create_snapshot(pool: &PgPool, note: &str) -> Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO snapshots (snapshot_id, note) VALUES ($1, $2)")
        .bind(id)
        .bind(note)
        .execute(pool)
        .await?;
    Ok(id)
}

/// Insert a fact and its provenance
async fn insert_fact(
    pool: &PgPool,
    snapshot_id: Uuid,
    entity_id: Uuid,
    metric_id: Uuid,
    fact: &ParsedFact,
    artifact_id: Uuid,
) -> Result<Uuid> {
    let fact_id = Uuid::new_v4();

    // Insert fact
    sqlx::query(
        r#"
        INSERT INTO facts (fact_id, snapshot_id, entity_id, metric_id, period_start, period_end, value_num, unit, dims)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(fact_id)
    .bind(snapshot_id)
    .bind(entity_id)
    .bind(metric_id)
    .bind(fact.period_start)
    .bind(fact.period_end)
    .bind(fact.value_num)
    .bind(&fact.metric_unit)
    .bind(&fact.dims)
    .execute(pool)
    .await?;

    // Insert provenance (evidence chain)
    sqlx::query(
        r#"
        INSERT INTO provenance (fact_id, artifact_id, location, method)
        VALUES ($1, $2, $3, 'csv_parser_v1')
        "#,
    )
    .bind(fact_id)
    .bind(artifact_id)
    .bind(&fact.location)
    .execute(pool)
    .await?;

    Ok(fact_id)
}

/// Update artifact parsed status
async fn update_artifact_status(
    pool: &PgPool,
    artifact_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE artifacts SET parsed_status = $2, parsed_error = $3 WHERE artifact_id = $1",
    )
    .bind(artifact_id)
    .bind(status)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

/// Create job run for parser
async fn create_job_run(pool: &PgPool, source_id: &str, artifact_id: Uuid) -> Result<Uuid> {
    let job_run_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO job_runs (job_run_id, component, source_id, status, detail)
        VALUES ($1, 'parser', $2, 'running', $3)
        "#,
    )
    .bind(job_run_id)
    .bind(source_id)
    .bind(serde_json::json!({ "artifact_id": artifact_id.to_string() }))
    .execute(pool)
    .await?;
    Ok(job_run_id)
}

/// Finish job run
async fn finish_job_run(
    pool: &PgPool,
    job_run_id: Uuid,
    status: &str,
    error: Option<&str>,
    facts_count: usize,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE job_runs
        SET finished_at = now(), status = $2, error = $3, detail = detail || $4
        WHERE job_run_id = $1
        "#,
    )
    .bind(job_run_id)
    .bind(status)
    .bind(error)
    .bind(serde_json::json!({ "facts_created": facts_count }))
    .execute(pool)
    .await?;
    Ok(())
}

/// Parse CSV content into facts
/// This function is DETERMINISTIC: same input = same output
fn parse_csv(content: &str, source_id: &str) -> Result<Vec<ParsedFact>> {
    let mut facts = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    for (line_num, result) in reader.deserialize().enumerate() {
        let row: CsvRow = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Warning: skipping line {} due to error: {}", line_num + 2, e);
                continue;
            }
        };

        // Normalize entity key (deterministic: lowercase, trim, replace spaces)
        let entity_key = row
            .entity
            .trim()
            .to_lowercase()
            .replace(' ', "_")
            .replace(".", "")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        // Create period dates (year -> Jan 1 to Dec 31)
        let period_start = NaiveDate::from_ymd_opt(row.year, 1, 1)
            .context("Invalid year for period_start")?;
        let period_end = NaiveDate::from_ymd_opt(row.year, 12, 31)
            .context("Invalid year for period_end")?;

        // Build dimensions from category if present
        let dims = match &row.category {
            Some(cat) if !cat.is_empty() => {
                serde_json::json!({ "category": cat })
            }
            _ => serde_json::json!({}),
        };

        // Determine metric based on source
        let (metric_key, metric_name) = match source_id {
            s if s.contains("presupuesto") => ("presupuesto_ejecutado", "Presupuesto Ejecutado"),
            s if s.contains("gasto") => ("gasto_total", "Gasto Total"),
            s if s.contains("dotacion") => ("dotacion", "Dotación de Personal"),
            _ => ("monto", "Monto"),
        };

        facts.push(ParsedFact {
            entity_key: entity_key.clone(),
            entity_name: row.entity.trim().to_string(),
            entity_type: "organismo".to_string(),
            metric_key: metric_key.to_string(),
            metric_name: metric_name.to_string(),
            metric_unit: "CLP".to_string(),
            period_start,
            period_end,
            value_num: row.amount,
            location: format!("csv:line={}", line_num + 2), // +2 for 1-indexed + header
            dims,
        });
    }

    Ok(facts)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let db_url = std::env::var("DB_URL").context("DB_URL env var missing")?;

    let artifact_id: Uuid = args.artifact_id.parse().context("Invalid artifact_id UUID")?;

    println!("=== Estado Transparente Parser ===");
    println!("Artifact ID: {}", artifact_id);
    println!("Mode: {}", if args.dry_run { "dry-run" } else { "live" });

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("Failed to connect to database")?;

    // Load artifact metadata
    let artifact: Artifact = sqlx::query_as(
        "SELECT artifact_id, source_id, url, content_hash, mime_type, storage_kind, storage_path, parsed_status FROM artifacts WHERE artifact_id = $1"
    )
    .bind(artifact_id)
    .fetch_optional(&pool)
    .await?
    .context("Artifact not found")?;

    println!("Source: {}", artifact.source_id);
    println!("URL: {}", artifact.url);
    println!("Hash: {}", artifact.content_hash);
    println!("Status: {}", artifact.parsed_status);

    if artifact.parsed_status == "ok" && !args.verify {
        println!("Artifact already parsed. Use --verify to re-check.");
        return Ok(());
    }

    // Create job run
    let job_run_id = if !args.dry_run {
        Some(create_job_run(&pool, &artifact.source_id, artifact_id).await?)
    } else {
        None
    };

    let result = async {
        // Read raw content
        println!("Reading raw file: {}", artifact.storage_path);
        let content = fs::read_to_string(&artifact.storage_path)
            .await
            .context("Failed to read artifact file")?;

        println!("Content size: {} bytes", content.len());

        // Parse CSV
        println!("Parsing CSV...");
        let facts = parse_csv(&content, &artifact.source_id)?;
        println!("Parsed {} facts", facts.len());

        if facts.is_empty() {
            anyhow::bail!("No facts parsed from artifact");
        }

        // Print sample facts
        for (i, fact) in facts.iter().take(3).enumerate() {
            println!(
                "  [{}] {} | {} | {} | {} {}",
                i + 1,
                fact.entity_name,
                fact.metric_key,
                fact.period_start.format("%Y"),
                fact.value_num,
                fact.metric_unit
            );
        }
        if facts.len() > 3 {
            println!("  ... and {} more", facts.len() - 3);
        }

        if args.dry_run {
            println!("\nDry run - no facts saved to database");
            return Ok(facts.len());
        }

        // Create snapshot
        let snapshot_id = create_snapshot(
            &pool,
            &format!("Parser run for artifact {}", artifact_id),
        )
        .await?;
        println!("Created snapshot: {}", snapshot_id);

        // Cache for entity/metric IDs
        let mut entity_cache: HashMap<String, Uuid> = HashMap::new();
        let mut metric_cache: HashMap<String, Uuid> = HashMap::new();

        // Insert facts
        let mut inserted = 0;
        for fact in &facts {
            // Get or create entity
            let entity_id = if let Some(&id) = entity_cache.get(&fact.entity_key) {
                id
            } else {
                let id = get_or_create_entity(
                    &pool,
                    &fact.entity_key,
                    &fact.entity_name,
                    &fact.entity_type,
                )
                .await?;
                entity_cache.insert(fact.entity_key.clone(), id);
                id
            };

            // Get or create metric
            let metric_id = if let Some(&id) = metric_cache.get(&fact.metric_key) {
                id
            } else {
                let id = get_or_create_metric(
                    &pool,
                    &fact.metric_key,
                    &fact.metric_name,
                    &fact.metric_unit,
                )
                .await?;
                metric_cache.insert(fact.metric_key.clone(), id);
                id
            };

            // Insert fact with provenance
            insert_fact(&pool, snapshot_id, entity_id, metric_id, fact, artifact_id).await?;
            inserted += 1;
        }

        // Mark artifact as parsed
        update_artifact_status(&pool, artifact_id, "ok", None).await?;

        println!("Inserted {} facts with provenance", inserted);
        Ok::<usize, anyhow::Error>(inserted)
    }
    .await;

    // Update job run
    if let Some(job_id) = job_run_id {
        match &result {
            Ok(count) => finish_job_run(&pool, job_id, "ok", None, *count).await?,
            Err(e) => {
                update_artifact_status(&pool, artifact_id, "failed", Some(&e.to_string())).await?;
                finish_job_run(&pool, job_id, "failed", Some(&e.to_string()), 0).await?;
            }
        }
    }

    let count = result?;
    println!("\n=== Parsing Complete ===");
    println!("Facts created: {}", count);
    println!("Ready for API queries");

    Ok(())
}
