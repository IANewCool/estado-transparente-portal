#!/usr/bin/env bash
set -euo pipefail

DB_URL="${DB_URL:-postgres://et:et@localhost:5432/et}"
MIG_DIR="$(cd "$(dirname "$0")"/../shared/sql/migrations && pwd)"

echo "Applying migrations from: $MIG_DIR"
for f in $(ls -1 "$MIG_DIR"/*.sql | sort); do
  echo "==> $f"
  psql "$DB_URL" -v ON_ERROR_STOP=1 -f "$f"
done
echo "Migrations applied."
