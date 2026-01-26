//! API Service - Public API for Estado Transparente
//!
//! Endpoints:
//! - GET /health - Health check
//! - GET /metrics - List all metrics
//! - GET /entities - Search/list entities
//! - GET /facts - Query facts with filters
//! - GET /compare - Compare facts between years
//! - GET /evidence - Get evidence for a fact

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    version: &'static str,
}

#[derive(Serialize, sqlx::FromRow)]
struct MetricResponse {
    metric_id: Uuid,
    metric_key: String,
    display_name: String,
    unit: String,
    description: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct EntityResponse {
    entity_id: Uuid,
    entity_key: String,
    display_name: String,
    entity_type: String,
}

#[derive(Serialize)]
struct FactResponse {
    fact_id: Uuid,
    entity_id: Uuid,
    entity_name: String,
    metric_id: Uuid,
    metric_name: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
    value_num: f64,
    unit: String,
    dims: serde_json::Value,
}

#[derive(Serialize)]
struct CompareRow {
    entity_id: Uuid,
    entity_name: String,
    metric_id: Uuid,
    metric_name: String,
    year_a: i32,
    value_a: Option<f64>,
    year_b: i32,
    value_b: Option<f64>,
    delta: Option<f64>,
    pct_change: Option<f64>,
    fact_id_a: Option<Uuid>,
    fact_id_b: Option<Uuid>,
}

#[derive(Serialize)]
struct CompareResponse {
    year_a: i32,
    year_b: i32,
    metric_id: Uuid,
    rows: Vec<CompareRow>,
}

#[derive(Serialize)]
struct EvidenceResponse {
    fact_id: Uuid,
    artifact: ArtifactInfo,
    location: Option<String>,
    method: String,
}

#[derive(Serialize)]
struct ArtifactInfo {
    artifact_id: Uuid,
    url: String,
    captured_at: DateTime<Utc>,
    content_hash: String,
    mime_type: String,
    size_bytes: i64,
    download_path: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Dashboard response types
#[derive(Serialize)]
struct DashboardResponse {
    year: i32,
    total_budget: i64,
    total_formatted: String,
    previous_year: Option<i32>,
    previous_total: Option<i64>,
    yoy_change_pct: Option<f64>,
    entities: Vec<DashboardEntity>,
    available_years: Vec<i32>,
}

#[derive(Serialize)]
struct DashboardEntity {
    entity_id: Uuid,
    entity_key: String,
    display_name: String,
    budget: i64,
    budget_formatted: String,
    percentage: f64,
}

// ============================================================================
// Query params
// ============================================================================

#[derive(Deserialize)]
struct EntitiesQuery {
    query: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct FactsQuery {
    metric_id: Option<Uuid>,
    entity_id: Option<Uuid>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct CompareQuery {
    metric_id: Uuid,
    entity_id: Option<Uuid>,
    year_a: i32,
    year_b: i32,
}

#[derive(Deserialize)]
struct DashboardQuery {
    year: Option<i32>,
}

#[derive(Deserialize)]
struct EvidenceQuery {
    fact_id: Uuid,
}

// ============================================================================
// Handlers
// ============================================================================

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: "0.1.0",
    })
}

async fn dashboard_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DashboardQuery>,
) -> impl IntoResponse {
    // Get available years
    let years_result: Result<Vec<(i32,)>, _> = sqlx::query_as(
        r#"
        SELECT DISTINCT EXTRACT(YEAR FROM period_start)::int as year
        FROM facts
        ORDER BY year DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await;

    let available_years: Vec<i32> = match years_result {
        Ok(rows) => rows.into_iter().map(|(y,)| y).collect(),
        Err(_) => vec![],
    };

    if available_years.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No data available".to_string(),
            }),
        )
            .into_response();
    }

    // Default to most recent year
    let year = params.year.unwrap_or_else(|| available_years[0]);
    let previous_year = if available_years.contains(&(year - 1)) {
        Some(year - 1)
    } else {
        None
    };

    // Get entities with budget for selected year
    let entities_result: Result<Vec<_>, _> = sqlx::query(
        r#"
        SELECT
            e.entity_id,
            e.entity_key,
            e.display_name,
            f.value_num as budget
        FROM facts f
        JOIN entities e ON f.entity_id = e.entity_id
        JOIN metrics m ON f.metric_id = m.metric_id
        WHERE m.metric_key = 'presupuesto_ley'
          AND EXTRACT(YEAR FROM f.period_start) = $1
        ORDER BY f.value_num DESC
        "#,
    )
    .bind(year)
    .fetch_all(&state.pool)
    .await;

    let entities = match entities_result {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    use sqlx::Row;

    // Calculate total (value_num is stored as FLOAT8 in PostgreSQL)
    let total_budget: i64 = entities
        .iter()
        .map(|r| r.get::<f64, _>("budget") as i64)
        .sum();

    // Get previous year total if available
    let previous_total: Option<i64> = if let Some(prev_year) = previous_year {
        let prev_result: Result<Option<(i64,)>, _> = sqlx::query_as(
            r#"
            SELECT SUM(f.value_num)::bigint as total
            FROM facts f
            JOIN metrics m ON f.metric_id = m.metric_id
            WHERE m.metric_key = 'presupuesto_ley'
              AND EXTRACT(YEAR FROM f.period_start) = $1
            "#,
        )
        .bind(prev_year)
        .fetch_optional(&state.pool)
        .await;

        prev_result.ok().flatten().map(|(t,)| t)
    } else {
        None
    };

    // Calculate YoY change
    let yoy_change_pct = match (previous_total, total_budget) {
        (Some(prev), total) if prev > 0 => {
            Some(((total - prev) as f64 / prev as f64) * 100.0)
        }
        _ => None,
    };

    // Format total for display
    let total_formatted = format_clp(total_budget);

    // Build entity list with percentages
    let dashboard_entities: Vec<DashboardEntity> = entities
        .iter()
        .map(|r| {
            let budget: i64 = r.get::<f64, _>("budget") as i64;
            let percentage = if total_budget > 0 {
                (budget as f64 / total_budget as f64) * 100.0
            } else {
                0.0
            };

            DashboardEntity {
                entity_id: r.get("entity_id"),
                entity_key: r.get("entity_key"),
                display_name: r.get("display_name"),
                budget,
                budget_formatted: format_clp(budget),
                percentage,
            }
        })
        .collect();

    Json(DashboardResponse {
        year,
        total_budget,
        total_formatted,
        previous_year,
        previous_total,
        yoy_change_pct,
        entities: dashboard_entities,
        available_years,
    })
    .into_response()
}

/// Format number as Chilean pesos
fn format_clp(amount: i64) -> String {
    let billions = amount as f64 / 1_000_000_000_000.0;
    if billions >= 1.0 {
        format!("${:.2} billones", billions)
    } else {
        let millions = amount as f64 / 1_000_000_000.0;
        format!("${:.1} mil millones", millions)
    }
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics: Result<Vec<MetricResponse>, _> = sqlx::query_as(
        "SELECT metric_id, metric_key, display_name, unit, description FROM metrics ORDER BY display_name",
    )
    .fetch_all(&state.pool)
    .await;

    match metrics {
        Ok(m) => Json(serde_json::json!({ "metrics": m })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn entities_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EntitiesQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100).min(1000);

    let entities: Result<Vec<EntityResponse>, _> = if let Some(q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        sqlx::query_as(
            r#"
            SELECT entity_id, entity_key, display_name, entity_type
            FROM entities
            WHERE LOWER(display_name) LIKE $1 OR LOWER(entity_key) LIKE $1
            ORDER BY display_name
            LIMIT $2
            "#,
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT entity_id, entity_key, display_name, entity_type FROM entities ORDER BY display_name LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    };

    match entities {
        Ok(e) => Json(serde_json::json!({ "entities": e })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn facts_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FactsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100).min(1000);

    // Build dynamic query
    let mut query = String::from(
        r#"
        SELECT f.fact_id, f.entity_id, e.display_name as entity_name,
               f.metric_id, m.display_name as metric_name,
               f.period_start, f.period_end, f.value_num, f.unit, f.dims
        FROM facts f
        JOIN entities e ON f.entity_id = e.entity_id
        JOIN metrics m ON f.metric_id = m.metric_id
        WHERE 1=1
        "#,
    );

    let mut bindings: Vec<String> = Vec::new();
    let mut idx = 1;

    if params.metric_id.is_some() {
        query.push_str(&format!(" AND f.metric_id = ${}", idx));
        idx += 1;
    }
    if params.entity_id.is_some() {
        query.push_str(&format!(" AND f.entity_id = ${}", idx));
        idx += 1;
    }
    if params.from.is_some() {
        query.push_str(&format!(" AND f.period_start >= ${}", idx));
        idx += 1;
    }
    if params.to.is_some() {
        query.push_str(&format!(" AND f.period_end <= ${}", idx));
        idx += 1;
    }

    query.push_str(&format!(" ORDER BY f.period_start DESC LIMIT ${}", idx));

    // Execute with bindings
    let mut q = sqlx::query(&query);

    if let Some(mid) = params.metric_id {
        q = q.bind(mid);
    }
    if let Some(eid) = params.entity_id {
        q = q.bind(eid);
    }
    if let Some(from) = params.from {
        q = q.bind(from);
    }
    if let Some(to) = params.to {
        q = q.bind(to);
    }
    q = q.bind(limit);

    let rows = q.fetch_all(&state.pool).await;

    match rows {
        Ok(rows) => {
            let facts: Vec<FactResponse> = rows
                .iter()
                .map(|row| {
                    use sqlx::Row;
                    FactResponse {
                        fact_id: row.get("fact_id"),
                        entity_id: row.get("entity_id"),
                        entity_name: row.get("entity_name"),
                        metric_id: row.get("metric_id"),
                        metric_name: row.get("metric_name"),
                        period_start: row.get("period_start"),
                        period_end: row.get("period_end"),
                        value_num: row.get("value_num"),
                        unit: row.get("unit"),
                        dims: row.get("dims"),
                    }
                })
                .collect();
            Json(serde_json::json!({ "facts": facts })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn compare_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CompareQuery>,
) -> impl IntoResponse {
    // Get facts for year A
    let year_a_start = NaiveDate::from_ymd_opt(params.year_a, 1, 1).unwrap();
    let year_a_end = NaiveDate::from_ymd_opt(params.year_a, 12, 31).unwrap();
    let year_b_start = NaiveDate::from_ymd_opt(params.year_b, 1, 1).unwrap();
    let year_b_end = NaiveDate::from_ymd_opt(params.year_b, 12, 31).unwrap();

    let query = if params.entity_id.is_some() {
        r#"
        WITH year_a AS (
            SELECT f.fact_id, f.entity_id, e.display_name as entity_name, f.value_num
            FROM facts f
            JOIN entities e ON f.entity_id = e.entity_id
            WHERE f.metric_id = $1 AND f.entity_id = $5
              AND f.period_start >= $2 AND f.period_end <= $3
        ),
        year_b AS (
            SELECT f.fact_id, f.entity_id, f.value_num
            FROM facts f
            WHERE f.metric_id = $1 AND f.entity_id = $5
              AND f.period_start >= $4 AND f.period_end <= $6
        )
        SELECT
            COALESCE(a.entity_id, b.entity_id) as entity_id,
            COALESCE(a.entity_name, e.display_name) as entity_name,
            a.value_num as value_a,
            b.value_num as value_b,
            a.fact_id as fact_id_a,
            b.fact_id as fact_id_b
        FROM year_a a
        FULL OUTER JOIN year_b b ON a.entity_id = b.entity_id
        LEFT JOIN entities e ON b.entity_id = e.entity_id
        ORDER BY entity_name
        "#
    } else {
        r#"
        WITH year_a AS (
            SELECT f.fact_id, f.entity_id, e.display_name as entity_name, f.value_num
            FROM facts f
            JOIN entities e ON f.entity_id = e.entity_id
            WHERE f.metric_id = $1
              AND f.period_start >= $2 AND f.period_end <= $3
        ),
        year_b AS (
            SELECT f.fact_id, f.entity_id, f.value_num
            FROM facts f
            WHERE f.metric_id = $1
              AND f.period_start >= $4 AND f.period_end <= $5
        )
        SELECT
            COALESCE(a.entity_id, b.entity_id) as entity_id,
            COALESCE(a.entity_name, e.display_name) as entity_name,
            a.value_num as value_a,
            b.value_num as value_b,
            a.fact_id as fact_id_a,
            b.fact_id as fact_id_b
        FROM year_a a
        FULL OUTER JOIN year_b b ON a.entity_id = b.entity_id
        LEFT JOIN entities e ON b.entity_id = e.entity_id
        ORDER BY entity_name
        "#
    };

    let rows = if let Some(eid) = params.entity_id {
        sqlx::query(query)
            .bind(params.metric_id)
            .bind(year_a_start)
            .bind(year_a_end)
            .bind(year_b_start)
            .bind(eid)
            .bind(year_b_end)
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query(query)
            .bind(params.metric_id)
            .bind(year_a_start)
            .bind(year_a_end)
            .bind(year_b_start)
            .bind(year_b_end)
            .fetch_all(&state.pool)
            .await
    };

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let compare_rows: Vec<CompareRow> = rows
                .iter()
                .map(|row| {
                    let value_a: Option<f64> = row.get("value_a");
                    let value_b: Option<f64> = row.get("value_b");
                    let delta = match (value_a, value_b) {
                        (Some(a), Some(b)) => Some(b - a),
                        _ => None,
                    };
                    let pct_change = match (value_a, value_b) {
                        (Some(a), Some(b)) if a != 0.0 => Some(((b - a) / a) * 100.0),
                        _ => None,
                    };

                    CompareRow {
                        entity_id: row.get("entity_id"),
                        entity_name: row.get("entity_name"),
                        metric_id: params.metric_id,
                        metric_name: String::new(), // Will be filled by frontend
                        year_a: params.year_a,
                        value_a,
                        year_b: params.year_b,
                        value_b,
                        delta,
                        pct_change,
                        fact_id_a: row.get("fact_id_a"),
                        fact_id_b: row.get("fact_id_b"),
                    }
                })
                .collect();

            Json(CompareResponse {
                year_a: params.year_a,
                year_b: params.year_b,
                metric_id: params.metric_id,
                rows: compare_rows,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn evidence_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EvidenceQuery>,
) -> impl IntoResponse {
    let result: Result<Option<_>, _> = sqlx::query(
        r#"
        SELECT
            p.fact_id,
            p.location,
            p.method,
            a.artifact_id,
            a.url,
            a.captured_at,
            a.content_hash,
            a.mime_type,
            a.size_bytes,
            a.storage_path
        FROM provenance p
        JOIN artifacts a ON p.artifact_id = a.artifact_id
        WHERE p.fact_id = $1
        "#,
    )
    .bind(params.fact_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(row)) => {
            use sqlx::Row;
            let storage_path: String = row.get("storage_path");
            let artifact_id: Uuid = row.get("artifact_id");

            Json(EvidenceResponse {
                fact_id: params.fact_id,
                artifact: ArtifactInfo {
                    artifact_id,
                    url: row.get("url"),
                    captured_at: row.get("captured_at"),
                    content_hash: row.get("content_hash"),
                    mime_type: row.get("mime_type"),
                    size_bytes: row.get("size_bytes"),
                    download_path: format!("/raw/{}", artifact_id),
                },
                location: row.get("location"),
                method: row.get("method"),
            })
            .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Evidence not found for fact".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DB_URL").context("DB_URL env var missing")?;
    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    println!("=== Estado Transparente API ===");
    println!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .context("Failed to connect to database")?;

    println!("Database connected");

    let state = Arc::new(AppState { pool });

    // CORS for web frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/metrics", get(metrics_handler))
        .route("/entities", get(entities_handler))
        .route("/facts", get(facts_handler))
        .route("/compare", get(compare_handler))
        .route("/evidence", get(evidence_handler))
        .layer(cors)
        .with_state(state);

    println!("API listening on http://{}", bind);
    println!("\nEndpoints:");
    println!("  GET /health");
    println!("  GET /metrics");
    println!("  GET /entities?query=&limit=");
    println!("  GET /facts?metric_id=&entity_id=&from=&to=&limit=");
    println!("  GET /compare?metric_id=&year_a=&year_b=&entity_id=");
    println!("  GET /evidence?fact_id=");

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
