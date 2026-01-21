# PROMPT MAESTRO — Claude Code (Estado Transparente)

Eres Claude Code trabajando dentro de este repositorio. Tu objetivo es convertir este scaffold en un MVP funcional
con **trazabilidad total**: ningún dato se muestra sin evidencia verificable.

## Principios no negociables (PLNN)
1) **LLM NO inventa datos**. Solo ayuda a escribir código, tests, docs. La verdad la determina el pipeline + evidencia.
2) Cada `fact` debe tener `evidence_ptr` (artifact hash + url + captured_at + location).
3) Todo cambio relevante debe quedar en commits pequeños y descriptivos.
4) Determinismo: misma entrada (artifact) + misma versión parser => misma salida.

## Alcance MVP (30 días)
- 1 a 3 fuentes (empezar por una: CSV/JSON si existe; si no, HTML/PDF con extracción simple)
- Pipeline completo: collector -> raw_store -> parser -> canonical DB -> API -> web
- Comparador por años para 1 métrica (ej: montos por categoría)

## Tareas (orden recomendado)

### A) Infra y esquema de base de datos
1) Revisa `shared/sql/migrations/001_init.sql` y complétalo si falta algo.
2) Asegura que `infra/docker-compose.yml` levanta:
   - Postgres (db)
   - MinIO (raw artifacts)
3) Implementa `infra/db_migrate.sh` para aplicar migraciones en orden.

### B) Collector (services/collector)
1) Implementa un conector de ejemplo (HTTP GET) con:
   - rate limit
   - cache local
   - guardado a MinIO (o filesystem si `RAW_STORE=fs`)
2) Debe escribir:
   - `artifacts` en DB (url, captured_at, content_hash, mime_type, size_bytes)
   - el archivo raw en el raw-store con nombre `artifact_id` o `hash`

### C) Parser (services/parser)
1) Crea un parser de ejemplo para una fuente simple (CSV):
   - Lee artifact raw
   - Normaliza a `entities`, `metrics`, `facts`
   - Valida montos/fechas
2) Inserta:
   - `facts`
   - `provenance` (fact_id -> artifact_id + location)
3) Si algo falla, deja el artifact marcado como `parsed_status=failed` con error.

### D) API (services/api)
1) Endpoints mínimos:
   - GET `/health`
   - GET `/metrics`
   - GET `/entities?query=`
   - GET `/facts?metric_id=&entity_id=&from=&to=&snapshot_id=`
   - GET `/compare?metric_id=&entity_id=&year_a=&year_b=`
   - GET `/evidence?fact_id=...`
2) El endpoint `/evidence` debe devolver:
   - metadata del artifact (url, captured_at, hash)
   - location (si existe)
   - link de descarga del raw (presigned si MinIO)

### E) Web (apps/web)
1) UI mínima:
   - Buscador de entidad
   - Selector de métrica
   - Selector de años A vs B
   - Tabla “Año A / Año B / Δ / %”
   - Botón “Ver evidencia” por fila (abre modal)
2) PWA:
   - cache básico de assets
   - (opcional) cache de la última comparación

### F) Observabilidad y auditoría
1) Tablas:
   - job_runs
   - data_quality (opcional)
2) Logs estructurados (JSON) para collector/parser/api.

## Definiciones y contratos
- JSON schemas en `shared/schema/*.json`
- El “contrato de verdad”:
  - Cada `fact` debe ser rastreable a un `artifact` por `provenance`.

## Entregables
- MVP corriendo con `docker compose up -d` + `cargo run` + `npm run dev`
- 1 fuente ingerida y visible en la UI con comparador por años
- README actualizado con comandos exactos

## Estándares
- Rust: clippy + fmt
- Tests mínimos para parser
- No agregar dependencias pesadas sin necesidad.
