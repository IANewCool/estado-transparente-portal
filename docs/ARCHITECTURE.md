# Arquitectura — Estado Transparente

> "La transparencia no se declara, se demuestra."

Este documento define la arquitectura completa del portal, su mapa de componentes, flujo de datos y ruta de desarrollo.

---

## 1. Visión General

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ESTADO TRANSPARENTE                                  │
│                    Portal de Transparencia Fiscal                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  "Datos públicos, verificables, sin interpretación editorial"               │
└─────────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────┐
                              │  CIUDADANO  │
                              └──────┬──────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CAPA PÚBLICA                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Portal    │  │    API      │  │  Descargas  │  │   Embeds    │        │
│  │    Web      │  │   REST      │  │    CSV      │  │   iFrame    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             CAPA DE DATOS                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │    Facts    │  │  Entities   │  │   Metrics   │  │ Provenance  │        │
│  │  (hechos)   │  │ (entidades) │  │  (métricas) │  │ (evidencia) │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CAPA DE INGESTA                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  Collector  │  │   Parser    │  │  Artifacts  │  │   Storage   │        │
│  │ (descarga)  │  │  (parseo)   │  │  (archivos) │  │   (MinIO)   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FUENTES OFICIALES                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   DIPRES    │  │ ChileCompra │  │ Contraloría │  │    SII      │        │
│  │ Presupuesto │  │   Compras   │  │   Gastos    │  │  Tributos   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Principio Central

**Nada se muestra si no tiene evidencia verificable.**

Para cada `fact` debe existir:
- `provenance.fact_id` → `artifacts.artifact_id`
- `artifacts.url` (fuente original)
- `artifacts.captured_at` (fecha de captura)
- `artifacts.content_hash` (SHA-256 verificable)
- `provenance.location` (página/tabla/fila en el archivo)

---

## 3. Mapa de Componentes

### 3.1 Estructura de Directorios

```
estado-transparente-portal/
│
├── services/                    # Backend Rust
│   ├── collector/               # Descarga artefactos
│   │   ├── src/main.rs         # CLI: --source-id --url
│   │   └── Cargo.toml          # reqwest, sha2, sqlx
│   │
│   ├── parser/                  # Transforma → facts
│   │   ├── src/main.rs         # CLI: --artifact-id
│   │   └── Cargo.toml          # csv, calamine, sqlx
│   │
│   └── api/                     # API REST
│       ├── src/main.rs         # Axum server :8080
│       └── Cargo.toml          # axum, sqlx, tower
│
├── apps/                        # Frontend
│   └── web/                     # React + Vite
│       ├── src/
│       │   ├── App.tsx         # Componente principal
│       │   ├── components/     # UI components
│       │   └── lib/            # API client, utils
│       ├── index.html
│       └── vite.config.ts
│
├── shared/                      # Compartido
│   └── sql/migrations/
│       └── 001_init.sql        # Schema canónico
│
├── config/                      # Configuración
│   └── sources.json            # Fuentes declaradas
│
├── infra/                       # Infraestructura
│   └── docker-compose.yml      # PostgreSQL + MinIO
│
├── data/                        # Datos locales
│   └── raw/                    # Artifacts descargados
│
├── docs/                        # Documentación
│   ├── ARCHITECTURE.md         # Este documento
│   ├── SOURCES.md              # Fuentes de datos
│   ├── ROADMAP.md              # Ruta de desarrollo
│   └── ...
│
├── PRINCIPLES.md               # Principios del sistema
├── Cargo.toml                  # Workspace Rust
└── README.md
```

### 3.2 Servicios Backend

| Servicio | Responsabilidad | CLI |
|----------|-----------------|-----|
| **collector** | Descarga archivos, calcula hash, guarda artifact | `--source-id --url` |
| **parser** | Lee artifact, parsea, valida, inserta facts | `--artifact-id [--dry-run]` |
| **api** | Sirve facts, entities, evidence vía REST | Puerto 8080 |

### 3.3 Frontend

| Página | Función |
|--------|---------|
| `/` | Dashboard con resumen de presupuesto |
| `/entidades` | Lista de entidades (ministerios, partidas) |
| `/entidad/:id` | Detalle de entidad con histórico |
| `/comparar` | Comparación entre entidades/años |
| `/verificar/:fact_id` | Cadena de evidencia completa |

---

## 4. Modelo de Datos

### 4.1 Diagrama ER

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    snapshots    │     │    artifacts    │     │   job_runs      │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ snapshot_id PK  │     │ artifact_id PK  │     │ job_run_id PK   │
│ created_at      │     │ source_id       │     │ component       │
│ note            │     │ url             │     │ source_id       │
└────────┬────────┘     │ captured_at     │     │ started_at      │
         │              │ content_hash    │     │ finished_at     │
         │              │ mime_type       │     │ status          │
         │              │ size_bytes      │     │ detail JSONB    │
         │              │ storage_path    │     │ error           │
         │              │ parsed_status   │     └─────────────────┘
         │              └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│     facts       │     │   provenance    │
├─────────────────┤     ├─────────────────┤
│ fact_id PK      │◄────│ provenance_id PK│
│ snapshot_id FK  │     │ fact_id FK      │
│ entity_id FK    │     │ artifact_id FK  │
│ metric_id FK    │     │ location        │
│ period_start    │     │ method          │
│ period_end      │     │ created_at      │
│ value_num       │     └─────────────────┘
│ unit            │
│ dims JSONB      │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌─────────────────┐     ┌─────────────────┐
│    entities     │     │    metrics      │
├─────────────────┤     ├─────────────────┤
│ entity_id PK    │     │ metric_id PK    │
│ entity_key UK   │     │ metric_key UK   │
│ display_name    │     │ display_name    │
│ entity_type     │     │ unit            │
└─────────────────┘     │ description     │
                        └─────────────────┘
```

### 4.2 Tablas Principales

| Tabla | Propósito | Índices |
|-------|-----------|---------|
| `snapshots` | Agrupa facts de un parsing run | PK |
| `artifacts` | Archivos descargados con hash | PK, UK(content_hash) |
| `entities` | Organismos, ministerios, partidas | PK, UK(entity_key) |
| `metrics` | Tipos de medición (presupuesto, gasto) | PK, UK(metric_key) |
| `facts` | Hechos: valor + período + dimensiones | PK, IDX(metric,period), IDX(entity) |
| `provenance` | Enlace fact → artifact + location | PK, IDX(fact_id) |
| `job_runs` | Historial de ejecuciones | PK |

---

## 5. Flujo de Datos

### 5.1 Pipeline de Ingesta

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   FUENTE     │    │  COLLECTOR   │    │   PARSER     │    │     API      │
│   OFICIAL    │    │              │    │              │    │              │
├──────────────┤    ├──────────────┤    ├──────────────┤    ├──────────────┤
│              │    │              │    │              │    │              │
│  CSV/XLS     │───▶│  1. Fetch    │───▶│  4. Read     │───▶│  7. Serve    │
│  público     │    │  2. Hash     │    │  5. Validate │    │  8. Filter   │
│              │    │  3. Store    │    │  6. Insert   │    │  9. Paginate │
│              │    │              │    │              │    │              │
└──────────────┘    └──────┬───────┘    └──────┬───────┘    └──────────────┘
                           │                   │
                           ▼                   ▼
                    ┌──────────────┐    ┌──────────────┐
                    │   MinIO      │    │  PostgreSQL  │
                    │  artifacts/  │    │  facts, etc  │
                    └──────────────┘    └──────────────┘
```

### 5.2 Detalle del Pipeline

| Paso | Componente | Acción | Output |
|------|------------|--------|--------|
| 1 | Collector | HTTP GET URL | bytes |
| 2 | Collector | SHA-256(bytes) | hash |
| 3 | Collector | Write to MinIO/FS | storage_path |
| 4 | Collector | INSERT artifacts | artifact_id |
| 5 | Parser | Read artifact | content |
| 6 | Parser | Validate headers | OK/AMBIGUITY |
| 7 | Parser | Parse rows | ParsedFact[] |
| 8 | Parser | INSERT facts + provenance | fact_ids |
| 9 | Parser | UPDATE artifact.parsed_status | ok/failed |

### 5.3 Flujo de Verificación Ciudadana

```
CIUDADANO                PORTAL                  BACKEND                FUENTE
    │                      │                        │                      │
    │  1. Ve presupuesto   │                        │                      │
    │─────────────────────▶│                        │                      │
    │                      │  2. GET /facts         │                      │
    │                      │───────────────────────▶│                      │
    │                      │◀───────────────────────│                      │
    │  3. Muestra tabla    │                        │                      │
    │◀─────────────────────│                        │                      │
    │                      │                        │                      │
    │  4. Click "Verificar"│                        │                      │
    │─────────────────────▶│                        │                      │
    │                      │  5. GET /evidence/:id  │                      │
    │                      │───────────────────────▶│                      │
    │                      │◀───────────────────────│                      │
    │  6. Ve cadena:       │                        │                      │
    │     - URL original   │                        │                      │
    │     - Hash SHA-256   │                        │                      │
    │     - Location       │                        │                      │
    │◀─────────────────────│                        │                      │
    │                      │                        │                      │
    │  7. Descarga original│                        │                      │
    │────────────────────────────────────────────────────────────────────▶│
    │                      │                        │                      │
    │  8. Calcula hash     │                        │                      │
    │     localmente       │                        │                      │
    │                      │                        │                      │
    │  9. Compara con      │                        │                      │
    │     hash guardado    │                        │                      │
    │     ✓ VERIFICADO     │                        │                      │
```

---

## 6. Principios Técnicos

### 6.1 Determinismo (PRINCIPLES #1)

```rust
// BTreeMap para orden determinista
let mut aggregates: BTreeMap<String, Aggregate> = BTreeMap::new();

// Sort final
facts.sort_by(|a, b| a.entity_key.cmp(&b.entity_key));

// Mismo input = mismo output, SIEMPRE
```

### 6.2 Detención ante Ambigüedad (PRINCIPLES #3)

```rust
if headers != EXPECTED_HEADERS {
    anyhow::bail!("AMBIGUITY: Headers mismatch");
}

// Nunca:
// - Inferir columnas
// - Ignorar errores
// - Improvisar mapeos
```

### 6.3 Separación de Dominios (PRINCIPLES #4)

```
Parsers independientes por tipo de dato:

parse_dipres_ley_csv()       → Presupuesto de Ley (aprobado)
parse_dipres_ejecucion()     → Ejecución Presupuestaria (gastado)
parse_chilecompra()          → Órdenes de Compra
parse_contraloria()          → Personal y Remuneraciones

NUNCA mezclar datos de diferentes dominios en el mismo fact.
```

---

## 7. Stack Tecnológico

### 7.1 Backend

| Componente | Tecnología | Versión |
|------------|------------|---------|
| Lenguaje | Rust | 2021 edition |
| Web | Axum | 0.7 |
| Database | PostgreSQL | 16 |
| ORM | sqlx | 0.8 |
| HTTP | reqwest | 0.12 |
| CSV | csv | 1.x |
| Excel | calamine | 0.26 |
| Hashing | sha2 | 0.10 |

### 7.2 Frontend

| Componente | Tecnología | Versión |
|------------|------------|---------|
| Framework | React | 18 |
| Bundler | Vite | 5 |
| Language | TypeScript | 5 |
| Styling | Tailwind CSS | 3 |
| HTTP | fetch | native |

### 7.3 Infraestructura

| Componente | Tecnología |
|------------|------------|
| Database | PostgreSQL (Docker) |
| Object Storage | MinIO (Docker) |
| Container | Docker Compose |
| CI/CD | GitHub Actions |

---

## 8. API REST

### 8.1 Endpoints

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/facts` | Lista facts con filtros |
| GET | `/facts/:id` | Detalle de un fact |
| GET | `/entities` | Lista de entidades |
| GET | `/entities/:id` | Detalle de entidad |
| GET | `/metrics` | Lista de métricas |
| GET | `/evidence/:fact_id` | Cadena de evidencia |
| GET | `/artifacts/:id/download` | Descarga artifact |

### 8.2 Filtros

```
GET /facts?entity_id=xxx&metric_id=yyy&year=2026&limit=100
GET /facts?period_start=2024-01-01&period_end=2026-12-31
```

### 8.3 Response Format

```json
{
  "facts": [
    {
      "fact_id": "uuid",
      "entity_id": "uuid",
      "entity_name": "IMPUESTOS",
      "metric_id": "uuid",
      "metric_name": "Presupuesto de Ley",
      "period_start": "2026-01-01",
      "period_end": "2026-12-31",
      "value_num": 519102002659000,
      "unit": "CLP",
      "dims": {
        "partida_code": "50",
        "aggregated_rows": 678
      }
    }
  ]
}
```

---

## 9. Testing

### 9.1 Estrategia

| Nivel | Cobertura | Herramienta |
|-------|-----------|-------------|
| Unit | Parsers, normalización | cargo test |
| Integration | DB operations | sqlx + test DB |
| E2E | Pipeline completo | Scripts |

### 9.2 Tests Obligatorios

```rust
#[test] fn test_determinism()           // Mismo input = mismo output
#[test] fn test_ambiguity_fails()       // Falla en headers incorrectos
#[test] fn test_provenance_complete()   // Cada fact tiene evidencia
```

---

## 10. Comandos Operativos

### 10.1 Setup

```bash
# Levantar infraestructura
docker compose -f infra/docker-compose.yml up -d

# Inicializar DB
docker exec -i infra-db-1 psql -U et -d et < shared/sql/migrations/001_init.sql

# Crear bucket MinIO
docker exec infra-minio-1 mc alias set local http://localhost:9000 minio minio123
docker exec infra-minio-1 mc mb local/et-raw
```

### 10.2 Ingesta

```bash
# Collector
cargo run --release -p collector -- \
  --source-id "dipres-ley-presupuestos-2026" \
  --url "https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv"

# Parser
cargo run --release -p parser -- \
  --artifact-id <UUID>
```

### 10.3 Desarrollo

```bash
# API
cargo run -p api

# Web
cd apps/web && npm run dev

# Tests
cargo test -p parser
```

---

## 11. Seguridad

### 11.1 Integridad de Datos

```
FUENTE → content_hash(SHA-256) → ARTIFACT → location → PROVENANCE → FACT
```

Cualquier modificación del archivo fuente invalida toda la cadena.

### 11.2 Consideraciones

- **SQL Injection**: Prevenido por sqlx prepared statements
- **XSS**: Prevenido por React escaping
- **Rate Limiting**: 1 req/sec a fuentes externas
- **No auth para lectura**: Datos públicos, API abierta
- **No datos personales**: Solo datos fiscales agregados

---

## 12. Escalabilidad Futura

### 12.1 Fase Actual (MVP)

- 1 fuente: DIPRES Ley 2026
- ~15K rows → 33 facts
- Single PostgreSQL

### 12.2 Fase 2 (6 meses)

- 5-10 fuentes
- ~1M rows
- Read replicas
- CDN para artifacts

### 12.3 Fase 3 (1+ año)

- 50+ fuentes
- Full-text search (Tantivy)
- Cache layer (Redis)
- Partitioned tables

---

*Última actualización: 2026-01-21*
*Versión: 2.0*
