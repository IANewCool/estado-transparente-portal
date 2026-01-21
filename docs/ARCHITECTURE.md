# Arquitectura — Estado Transparente

## Principio central
**Nada se muestra si no tiene evidencia verificable**.

## Flujo
1) **Collector** descarga un recurso público (PDF/HTML/CSV)
2) Guarda el archivo crudo como **artifact** (raw-store) y registra metadatos (hash, URL, fecha)
3) **Parser** transforma artifact -> datos canónicos deterministas
4) Inserta `facts` + `provenance` (evidencia)
5) **API** expone consultas y evidencia
6) **Web/PWA** permite buscar, comparar, y revisar fuente

## Componentes
- `services/collector`:
  - Connectors por fuente
  - rate limit, cache
  - escribe `artifacts`, `job_runs`
  - guarda raw en MinIO o FS

- `services/parser`:
  - Extractors/parsers
  - validaciones deterministas
  - escribe `facts`, `entities`, `metrics`, `provenance`

- `services/api`:
  - consultas (facts, compare)
  - evidencia (`/evidence`)
  - descarga raw (presigned o proxy)

- `apps/web`:
  - UI comparador por años
  - “ver evidencia” por dato

## Time Machine (snapshots)
- Cada corrida de ingesta/parsing produce `snapshot_id`
- Permite:
  - `latest`: vista más reciente
  - `as_of(date)`: cómo se veía en una fecha

## Contrato de evidencia (obligatorio)
Para cada `fact`:
- `provenance.fact_id -> artifacts.artifact_id`
- `artifacts.url`
- `artifacts.captured_at`
- `artifacts.content_hash`
- `provenance.location` (si se puede: página/tabla/fila)

## Seguridad / legal
- Portal independiente (no usar branding oficial)
- Respetar datos personales, anonimizar o excluir
- Cache y rate limit para no degradar sitios fuente
