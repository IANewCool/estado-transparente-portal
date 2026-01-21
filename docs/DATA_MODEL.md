# Modelo de datos (canónico)

## Entidades
- `entities`: organismos/servicios/proveedores/etc.

## Métricas
- `metrics`: definiciones (ej: “monto adjudicado”, “dotación”, “presupuesto ejecutado”)

## Hechos
- `facts`: valores numéricos por periodo y dimensiones
  - `entity_id`
  - `metric_id`
  - `period_start` / `period_end`
  - `value_num`
  - `unit`
  - `snapshot_id`

## Evidencia
- `artifacts`: archivos crudos + metadata
- `provenance`: relación `fact` -> `artifact` + location

## Snapshots
- `snapshots`: corrida/versionado para reproducibilidad

## Jobs
- `job_runs`: auditoría del pipeline
