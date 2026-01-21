//! Parser Service - Transforms raw artifacts into canonical facts
//!
//! Responsibilities:
//! - Load artifact metadata and raw content
//! - Parse CSV/XLS deterministically
//! - Upsert entities and metrics
//! - Insert facts with provenance (evidence chain)
//! - Mark artifact as parsed or failed
//!
//! CRITICAL: This service must be DETERMINISTIC
//! Same artifact + same parser version = same output

use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveDate;
use clap::Parser;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
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
#[derive(Debug, Clone, PartialEq)]
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

// =============================================================================
// XLS PARSER - DIPRES Presupuesto Format Only
// =============================================================================

/// Known DIPRES column mappings (explicit, not inferred)
/// These are the exact column names used in DIPRES budget files
const DIPRES_ENTITY_COLUMNS: &[&str] = &["partida", "capitulo", "programa", "servicio", "organismo"];
const DIPRES_YEAR_COLUMNS: &[&str] = &["año", "anio", "periodo"];
const DIPRES_AMOUNT_COLUMNS: &[&str] = &["monto", "presupuesto", "ppto_inicial", "ley_inicial", "total"];
const DIPRES_CATEGORY_COLUMNS: &[&str] = &["subtitulo", "item", "asignacion", "categoria"];

/// Column mapping result for DIPRES XLS
#[derive(Debug)]
struct DipresColumnMapping {
    entity_col: Option<usize>,
    entity_name: String,
    year_col: Option<usize>,
    year_name: String,
    amount_col: Option<usize>,
    amount_name: String,
    category_col: Option<usize>,
    category_name: String,
}

/// Find column index by matching against known column names
fn find_column(headers: &[String], candidates: &[&str]) -> Option<(usize, String)> {
    for (idx, header) in headers.iter().enumerate() {
        let normalized = header.trim().to_lowercase();
        for candidate in candidates {
            if normalized == *candidate || normalized.contains(candidate) {
                return Some((idx, header.clone()));
            }
        }
    }
    None
}

/// Parse DIPRES XLS file into facts
/// This function is DETERMINISTIC: same XLS file = same output
/// Only supports DIPRES budget format - not a general XLS parser
fn parse_dipres_xls(file_path: &Path, source_id: &str) -> Result<Vec<ParsedFact>> {
    println!("Opening XLS file: {}", file_path.display());

    // Open workbook (calamine auto-detects format: xls, xlsx, xlsb, ods)
    let mut workbook: calamine::Sheets<_> = open_workbook_auto(file_path)
        .context("Failed to open XLS file")?;

    // Get sheet names and use the first one
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        anyhow::bail!("XLS file has no sheets");
    }

    let sheet_name = &sheet_names[0];
    println!("Reading sheet: '{}' (first of {} sheets)", sheet_name, sheet_names.len());

    // Get the range (all cells in the sheet)
    let range = workbook
        .worksheet_range(sheet_name)
        .context("Failed to read sheet")?;

    let (row_count, col_count) = range.get_size();
    println!("Sheet size: {} rows x {} columns", row_count, col_count);

    if row_count < 2 {
        anyhow::bail!("Sheet has insufficient rows (need header + data)");
    }

    // Extract headers from first row
    let headers: Vec<String> = range
        .rows()
        .next()
        .context("No header row")?
        .iter()
        .map(|cell| match cell {
            Data::String(s) => s.trim().to_string(),
            Data::Empty => String::new(),
            other => format!("{}", other),
        })
        .collect();

    println!("\nDetected columns ({}):", headers.len());
    for (i, h) in headers.iter().enumerate() {
        if !h.is_empty() {
            println!("  [{:2}] {}", i, h);
        }
    }

    // Create column mapping using explicit DIPRES column names
    let mapping = DipresColumnMapping {
        entity_col: find_column(&headers, DIPRES_ENTITY_COLUMNS).map(|(i, _)| i),
        entity_name: find_column(&headers, DIPRES_ENTITY_COLUMNS)
            .map(|(_, n)| n)
            .unwrap_or_default(),
        year_col: find_column(&headers, DIPRES_YEAR_COLUMNS).map(|(i, _)| i),
        year_name: find_column(&headers, DIPRES_YEAR_COLUMNS)
            .map(|(_, n)| n)
            .unwrap_or_default(),
        amount_col: find_column(&headers, DIPRES_AMOUNT_COLUMNS).map(|(i, _)| i),
        amount_name: find_column(&headers, DIPRES_AMOUNT_COLUMNS)
            .map(|(_, n)| n)
            .unwrap_or_default(),
        category_col: find_column(&headers, DIPRES_CATEGORY_COLUMNS).map(|(i, _)| i),
        category_name: find_column(&headers, DIPRES_CATEGORY_COLUMNS)
            .map(|(_, n)| n)
            .unwrap_or_default(),
    };

    println!("\nColumn mapping:");
    println!("  Entity:   {} -> {:?}", mapping.entity_name, mapping.entity_col);
    println!("  Year:     {} -> {:?}", mapping.year_name, mapping.year_col);
    println!("  Amount:   {} -> {:?}", mapping.amount_name, mapping.amount_col);
    println!("  Category: {} -> {:?}", mapping.category_name, mapping.category_col);

    // Validate required columns
    let entity_col = mapping.entity_col.context(
        "AMBIGUITY: No entity column found. Expected one of: partida, capitulo, programa, servicio, organismo"
    )?;
    let amount_col = mapping.amount_col.context(
        "AMBIGUITY: No amount column found. Expected one of: monto, presupuesto, ppto_inicial, ley_inicial, total"
    )?;

    // Year column is optional - we may use a fixed year from source_id
    let fixed_year: Option<i32> = if mapping.year_col.is_none() {
        // Try to extract year from source_id (e.g., "dipres-presupuesto-ley-2024")
        source_id
            .split('-')
            .filter_map(|s| s.parse::<i32>().ok())
            .find(|&y| y >= 2000 && y <= 2100)
    } else {
        None
    };

    if mapping.year_col.is_none() && fixed_year.is_none() {
        anyhow::bail!(
            "AMBIGUITY: No year column found and cannot extract year from source_id '{}'",
            source_id
        );
    }

    println!("\nParsing data rows...");

    let mut facts = Vec::new();
    let mut skipped = 0;

    // Iterate over data rows (skip header)
    for (row_idx, row) in range.rows().enumerate().skip(1) {
        // Extract entity
        let entity = match row.get(entity_col) {
            Some(Data::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
            _ => {
                skipped += 1;
                continue;
            }
        };

        // Extract year
        let year: i32 = if let Some(year_col) = mapping.year_col {
            match row.get(year_col) {
                Some(Data::Float(f)) => *f as i32,
                Some(Data::Int(i)) => *i as i32,
                Some(Data::String(s)) => s.trim().parse().unwrap_or(0),
                _ => fixed_year.unwrap_or(0),
            }
        } else {
            fixed_year.unwrap_or(0)
        };

        if year < 2000 || year > 2100 {
            skipped += 1;
            continue;
        }

        // Extract amount
        let amount: f64 = match row.get(amount_col) {
            Some(Data::Float(f)) => *f,
            Some(Data::Int(i)) => *i as f64,
            Some(Data::String(s)) => s.trim().replace(",", "").replace(".", "").parse().unwrap_or(0.0),
            _ => {
                skipped += 1;
                continue;
            }
        };

        if amount == 0.0 {
            skipped += 1;
            continue;
        }

        // Extract category (optional)
        let category: Option<String> = mapping.category_col.and_then(|col| {
            match row.get(col) {
                Some(Data::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
                _ => None,
            }
        });

        // Normalize entity key (deterministic)
        let entity_key = entity
            .to_lowercase()
            .replace(' ', "_")
            .replace(".", "")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        // Create period dates
        let period_start = NaiveDate::from_ymd_opt(year, 1, 1)
            .context("Invalid year for period_start")?;
        let period_end = NaiveDate::from_ymd_opt(year, 12, 31)
            .context("Invalid year for period_end")?;

        // Build dimensions
        let dims = match &category {
            Some(cat) => serde_json::json!({ "category": cat }),
            None => serde_json::json!({}),
        };

        // Determine metric based on source
        let (metric_key, metric_name) = if source_id.contains("presupuesto") {
            ("presupuesto_ley", "Presupuesto de Ley")
        } else if source_id.contains("gasto") {
            ("gasto_ejecutado", "Gasto Ejecutado")
        } else {
            ("monto", "Monto")
        };

        facts.push(ParsedFact {
            entity_key,
            entity_name: entity,
            entity_type: "organismo".to_string(),
            metric_key: metric_key.to_string(),
            metric_name: metric_name.to_string(),
            metric_unit: "CLP".to_string(),
            period_start,
            period_end,
            value_num: amount,
            location: format!("xls:sheet='{}':row={}", sheet_name, row_idx + 1),
            dims,
        });
    }

    println!("Parsed {} facts, skipped {} rows", facts.len(), skipped);

    if facts.is_empty() {
        anyhow::bail!("No facts parsed from XLS file - check column mapping");
    }

    Ok(facts)
}

/// Detect if file is XLS/XLSX based on mime type or file signature
fn is_excel_file(mime_type: &str, storage_path: &str) -> bool {
    mime_type.contains("excel")
        || mime_type.contains("spreadsheet")
        || storage_path.ends_with(".xls")
        || storage_path.ends_with(".xlsx")
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
        // Detect file format and parse accordingly
        println!("Reading raw file: {}", artifact.storage_path);
        println!("MIME type: {}", artifact.mime_type);

        let facts = if is_excel_file(&artifact.mime_type, &artifact.storage_path) {
            // Parse as Excel (XLS/XLSX)
            println!("\nDetected Excel format - using DIPRES XLS parser");
            parse_dipres_xls(Path::new(&artifact.storage_path), &artifact.source_id)?
        } else {
            // Parse as CSV (default)
            let content = fs::read_to_string(&artifact.storage_path)
                .await
                .context("Failed to read artifact file")?;
            println!("Content size: {} bytes", content.len());
            println!("Parsing CSV...");
            parse_csv(&content, &artifact.source_id)?
        };

        println!("\nParsed {} facts total", facts.len());

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

// =============================================================================
// TESTS - Critical for ensuring DETERMINISM
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // -------------------------------------------------------------------------
    // DETERMINISM TESTS - Same input MUST produce same output
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_csv_determinism() {
        let csv = "entidad,categoria,anio,monto\nMinisterio de Salud,Personal,2024,1000000\n";

        let result1 = parse_csv(csv, "presupuesto-test").unwrap();
        let result2 = parse_csv(csv, "presupuesto-test").unwrap();

        assert_eq!(result1.len(), result2.len());
        assert_eq!(result1[0].entity_key, result2[0].entity_key);
        assert_eq!(result1[0].value_num, result2[0].value_num);
        assert_eq!(result1[0].period_start, result2[0].period_start);
    }

    #[test]
    fn test_parse_csv_determinism_multiple_runs() {
        let csv = r#"entidad,categoria,anio,monto
Ministerio de Educación,Personal,2024,1250000000000
Ministerio de Educación,Operaciones,2024,450000000000
Ministerio de Salud,Personal,2024,980000000000
"#;

        // Run 10 times and verify identical output
        let baseline = parse_csv(csv, "presupuesto").unwrap();
        for _ in 0..10 {
            let result = parse_csv(csv, "presupuesto").unwrap();
            assert_eq!(baseline.len(), result.len());
            for (a, b) in baseline.iter().zip(result.iter()) {
                assert_eq!(a.entity_key, b.entity_key);
                assert_eq!(a.metric_key, b.metric_key);
                assert_eq!(a.value_num, b.value_num);
                assert_eq!(a.location, b.location);
            }
        }
    }

    // -------------------------------------------------------------------------
    // ENTITY KEY NORMALIZATION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_key_normalization_basic() {
        let csv = "entidad,anio,monto\nMinisterio de Salud,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_key, "ministerio_de_salud");
    }

    #[test]
    fn test_entity_key_normalization_accents() {
        let csv = "entidad,anio,monto\nMinisterio de Educación,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_key, "ministerio_de_educación");
        assert_eq!(facts[0].entity_name, "Ministerio de Educación");
    }

    #[test]
    fn test_entity_key_normalization_dots_removed() {
        let csv = "entidad,anio,monto\nGob. Regional de Valparaíso,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_key, "gob_regional_de_valparaíso");
    }

    #[test]
    fn test_entity_key_normalization_special_chars() {
        let csv = "entidad,anio,monto\n\"Serv. Nacional (SERNAC)\",2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        // Only alphanumeric and underscore allowed
        assert!(!facts[0].entity_key.contains('('));
        assert!(!facts[0].entity_key.contains(')'));
    }

    #[test]
    fn test_entity_key_normalization_whitespace() {
        let csv = "entidad,anio,monto\n\"  Ministerio de Salud  \",2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_key, "ministerio_de_salud");
        assert_eq!(facts[0].entity_name, "Ministerio de Salud");
    }

    // -------------------------------------------------------------------------
    // METRIC DETECTION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_metric_detection_presupuesto() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "dipres-presupuesto-2024").unwrap();
        assert_eq!(facts[0].metric_key, "presupuesto_ejecutado");
        assert_eq!(facts[0].metric_name, "Presupuesto Ejecutado");
    }

    #[test]
    fn test_metric_detection_gasto() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "contraloria-gasto-2024").unwrap();
        assert_eq!(facts[0].metric_key, "gasto_total");
        assert_eq!(facts[0].metric_name, "Gasto Total");
    }

    #[test]
    fn test_metric_detection_dotacion() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "dipres-dotacion-2024").unwrap();
        assert_eq!(facts[0].metric_key, "dotacion");
        assert_eq!(facts[0].metric_name, "Dotación de Personal");
    }

    #[test]
    fn test_metric_detection_unknown() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "unknown-source").unwrap();
        assert_eq!(facts[0].metric_key, "monto");
        assert_eq!(facts[0].metric_name, "Monto");
    }

    // -------------------------------------------------------------------------
    // PERIOD DATE TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_period_dates_year_2024() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].period_start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(facts[0].period_end, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_period_dates_year_2025() {
        let csv = "entidad,anio,monto\nTest,2025,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].period_start, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(facts[0].period_end, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
    }

    // -------------------------------------------------------------------------
    // DIMENSIONS TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_dimensions_with_category() {
        let csv = "entidad,categoria,anio,monto\nTest,Personal,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].dims, serde_json::json!({"category": "Personal"}));
    }

    #[test]
    fn test_dimensions_without_category() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].dims, serde_json::json!({}));
    }

    #[test]
    fn test_dimensions_empty_category() {
        let csv = "entidad,categoria,anio,monto\nTest,,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].dims, serde_json::json!({}));
    }

    // -------------------------------------------------------------------------
    // LINE LOCATION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_line_location_first_row() {
        let csv = "entidad,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].location, "csv:line=2"); // Header is line 1
    }

    #[test]
    fn test_line_location_multiple_rows() {
        let csv = "entidad,anio,monto\nA,2024,1\nB,2024,2\nC,2024,3\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].location, "csv:line=2");
        assert_eq!(facts[1].location, "csv:line=3");
        assert_eq!(facts[2].location, "csv:line=4");
    }

    // -------------------------------------------------------------------------
    // VALUE PARSING TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_value_parsing_integer() {
        let csv = "entidad,anio,monto\nTest,2024,1000000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].value_num, 1000000.0);
    }

    #[test]
    fn test_value_parsing_large_number() {
        let csv = "entidad,anio,monto\nTest,2024,1250000000000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].value_num, 1250000000000.0);
    }

    #[test]
    fn test_value_parsing_decimal() {
        let csv = "entidad,anio,monto\nTest,2024,1234.56\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].value_num, 1234.56);
    }

    // -------------------------------------------------------------------------
    // COLUMN ALIAS TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_column_alias_entity() {
        let csv = "organismo,anio,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_name, "Test");
    }

    #[test]
    fn test_column_alias_year() {
        let csv = "entidad,periodo,monto\nTest,2024,1000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].period_start.year(), 2024);
    }

    #[test]
    fn test_column_alias_amount() {
        let csv = "entidad,anio,valor\nTest,2024,5000\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].value_num, 5000.0);
    }

    // -------------------------------------------------------------------------
    // EDGE CASES
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_csv() {
        let csv = "entidad,anio,monto\n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts.len(), 0);
    }

    #[test]
    fn test_whitespace_trimming() {
        let csv = "entidad,anio,monto\n  Test  ,  2024  ,  1000  \n";
        let facts = parse_csv(csv, "test").unwrap();
        assert_eq!(facts[0].entity_name, "Test");
        assert_eq!(facts[0].value_num, 1000.0);
    }

    #[test]
    fn test_multiple_entities_same_year() {
        let csv = r#"entidad,categoria,anio,monto
Ministerio A,Personal,2024,100
Ministerio A,Operaciones,2024,200
Ministerio B,Personal,2024,300
"#;
        let facts = parse_csv(csv, "presupuesto").unwrap();
        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].entity_key, "ministerio_a");
        assert_eq!(facts[1].entity_key, "ministerio_a");
        assert_eq!(facts[2].entity_key, "ministerio_b");
    }

    // -------------------------------------------------------------------------
    // REAL DATA FORMAT TESTS (DIPRES format)
    // -------------------------------------------------------------------------

    #[test]
    fn test_dipres_budget_format() {
        let csv = r#"entidad,categoria,anio,monto
Ministerio de Educación,Personal,2024,1250000000000
Ministerio de Educación,Operaciones,2024,450000000000
Ministerio de Educación,Inversión,2024,380000000000
Ministerio de Salud,Personal,2024,980000000000
"#;
        let facts = parse_csv(csv, "dipres-presupuesto-2024").unwrap();

        assert_eq!(facts.len(), 4);
        assert_eq!(facts[0].metric_key, "presupuesto_ejecutado");
        assert_eq!(facts[0].entity_key, "ministerio_de_educación");
        assert_eq!(facts[0].value_num, 1250000000000.0);
        assert_eq!(facts[0].dims["category"], "Personal");
    }
}
