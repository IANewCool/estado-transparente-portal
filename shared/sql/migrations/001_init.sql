-- 001_init.sql — canonical schema (Postgres)

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS snapshots (
  snapshot_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  note TEXT
);

CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_id TEXT NOT NULL,
  url TEXT NOT NULL,
  captured_at TIMESTAMPTZ NOT NULL,
  content_hash TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  size_bytes BIGINT NOT NULL,
  storage_kind TEXT NOT NULL DEFAULT 'minio', -- 'minio' | 'fs'
  storage_path TEXT NOT NULL,
  parsed_status TEXT NOT NULL DEFAULT 'pending', -- pending|ok|failed
  parsed_error TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_artifacts_hash ON artifacts(content_hash);

CREATE TABLE IF NOT EXISTS entities (
  entity_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_key TEXT UNIQUE NOT NULL,
  display_name TEXT NOT NULL,
  entity_type TEXT NOT NULL DEFAULT 'org'
);

CREATE TABLE IF NOT EXISTS metrics (
  metric_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  metric_key TEXT UNIQUE NOT NULL,
  display_name TEXT NOT NULL,
  unit TEXT NOT NULL DEFAULT 'CLP',
  description TEXT
);

CREATE TABLE IF NOT EXISTS facts (
  fact_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  snapshot_id UUID NOT NULL REFERENCES snapshots(snapshot_id) ON DELETE CASCADE,
  entity_id UUID NOT NULL REFERENCES entities(entity_id),
  metric_id UUID NOT NULL REFERENCES metrics(metric_id),
  period_start DATE NOT NULL,
  period_end DATE NOT NULL,
  value_num DOUBLE PRECISION NOT NULL,
  unit TEXT NOT NULL,
  dims JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_facts_metric_time ON facts(metric_id, period_start, period_end);
CREATE INDEX IF NOT EXISTS idx_facts_entity ON facts(entity_id);

CREATE TABLE IF NOT EXISTS provenance (
  provenance_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  fact_id UUID NOT NULL REFERENCES facts(fact_id) ON DELETE CASCADE,
  artifact_id UUID NOT NULL REFERENCES artifacts(artifact_id) ON DELETE CASCADE,
  location TEXT, -- e.g. "page=12; table=3; row=7" or "csv:line=42"
  method TEXT NOT NULL DEFAULT 'parse',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_prov_fact ON provenance(fact_id);

CREATE TABLE IF NOT EXISTS job_runs (
  job_run_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  component TEXT NOT NULL, -- collector|parser
  source_id TEXT NOT NULL,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'running', -- running|ok|failed
  detail JSONB NOT NULL DEFAULT '{}'::jsonb,
  error TEXT
);
