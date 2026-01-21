# Estado Transparente (Portal Ciudadano) — MVP v0.1

> Portal independiente para **consolidar información pública** del Estado de Chile en un solo lugar,
> con **comparación por años** y **trazabilidad/evidencia** (fuente + fecha + hash + versión).

**Versión:** 0.1.0 | **Fecha:** 2026-01-21

## Características

- **Ingesta** de fuentes públicas (CSV/JSON/HTML/PDF) con rate limit + cache
- **Normalización** a modelo canónico (`facts`, `entities`, `metrics`)
- **Evidencia verificable** por cada dato (artifact hash + URL + captured_at + ubicación)
- **API pública** para consultas y comparaciones
- **Web** para buscar, comparar años, y "ver evidencia"
- **Determinismo**: mismo input = mismo output, siempre

## Quickstart

### Requisitos

- Docker + Docker Compose
- Rust (stable) - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js 18+ - para el frontend
- PostgreSQL client (`psql`) - para scripts

### 1. Clonar y configurar

```bash
git clone https://github.com/estado-transparente/portal.git
cd portal
cp .env.example .env
```

### 2. Levantar infraestructura

```bash
cd infra
docker compose up -d
cd ..
```

### 3. Ejecutar demo pipeline

```bash
./scripts/demo_pipeline.sh
```

Esto:
- Aplica migraciones a PostgreSQL
- Carga datos de demo (presupuesto por ministerio)
- Parsea y crea facts con provenance

### 4. Iniciar API

```bash
cargo run --bin api
```

La API estará en `http://localhost:8080`

### 5. Iniciar Web

```bash
cd apps/web
npm install
npm run dev
```

El frontend estará en `http://localhost:5173`

### 6. Probar

1. Abre `http://localhost:5173`
2. Selecciona la métrica "Monto"
3. Configura años 2024 vs 2025
4. Click "Comparar"
5. Click "A" o "B" en cualquier fila para ver evidencia

---

## Documentación

### Técnica
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — Arquitectura del sistema
- [`docs/DATA_MODEL.md`](docs/DATA_MODEL.md) — Modelo de datos canónico
- [`docs/SOURCES.md`](docs/SOURCES.md) — Catálogo de fuentes

### Independencia y Financiamiento
- [`docs/INDEPENDENCIA_ALGORITMICA.md`](docs/INDEPENDENCIA_ALGORITMICA.md) — Garantías de neutralidad
- [`docs/FINANCIAMIENTO.md`](docs/FINANCIAMIENTO.md) — Modelo de financiamiento
- [`docs/PLANES_PRECIOS.md`](docs/PLANES_PRECIOS.md) — Planes Pro y precios

### Legal
- [`docs/LEGAL.md`](docs/LEGAL.md) — Consideraciones legales

---

## Estructura del Proyecto

```
estado-transparente-portal/
├── services/
│   ├── collector/     # Descarga artifacts (Rust)
│   ├── parser/        # Normaliza datos (Rust)
│   └── api/           # API REST (Rust/Axum)
├── apps/
│   └── web/           # Frontend (React/Vite)
├── shared/
│   ├── schema/        # JSON schemas
│   └── sql/           # Migraciones PostgreSQL
├── infra/             # Docker Compose
├── data/              # Datos de demo
├── scripts/           # Scripts de utilidad
└── docs/              # Documentación
```

---

## API Endpoints

| Endpoint | Descripción |
|----------|-------------|
| `GET /health` | Health check |
| `GET /metrics` | Lista métricas disponibles |
| `GET /entities?query=` | Busca entidades |
| `GET /facts?metric_id=&entity_id=&from=&to=` | Consulta facts |
| `GET /compare?metric_id=&year_a=&year_b=` | Compara años |
| `GET /evidence?fact_id=` | Obtiene evidencia de un fact |

---

## Pipeline de Datos

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  COLLECTOR  │ ──▶ │   PARSER    │ ──▶ │    API      │
│             │     │             │     │             │
│ • Descarga  │     │ • Normaliza │     │ • Consultas │
│ • Hash      │     │ • Valida    │     │ • Compare   │
│ • Almacena  │     │ • Provenance│     │ • Evidence  │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
   artifacts           facts              endpoints
   (raw files)      (canonical)          (JSON API)
```

---

## Independencia

Este portal es **100% independiente**:

- ❌ No recibimos financiamiento del Estado
- ❌ No aceptamos dinero de entidades fiscalizadas
- ❌ No hay publicidad política
- ✅ El motor es determinista y auditable
- ✅ El código es open source
- ✅ Las finanzas son públicas

**El dinero no toca los datos.** Ver [`docs/INDEPENDENCIA_ALGORITMICA.md`](docs/INDEPENDENCIA_ALGORITMICA.md).

---

## Desarrollo

### Compilar todo

```bash
cargo build --release
```

### Ejecutar tests

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt
```

### Agregar nueva fuente

1. Documentar en `docs/SOURCES.md`
2. Crear parser si es necesario
3. Ejecutar collector: `cargo run --bin collector -- --source-id X --url "..."`
4. Ejecutar parser: `cargo run --bin parser -- --artifact-id <UUID>`

---

## Licencia

- **Código:** MIT
- **Datos:** Derivados de fuentes públicas con atribución y enlaces a la fuente original
- **Documentación:** CC BY 4.0

---

## Contribuir

1. Fork del repositorio
2. Crea rama: `git checkout -b feature/mi-feature`
3. Commit: `git commit -m "feat(módulo): descripción"`
4. Push: `git push origin feature/mi-feature`
5. Pull Request

---

*"La transparencia no se declara, se demuestra."*
