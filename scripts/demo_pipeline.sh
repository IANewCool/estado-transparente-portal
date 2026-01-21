#!/usr/bin/env bash
# Estado Transparente - Demo Pipeline
# Este script carga datos de demo para probar el sistema completo

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Load environment
if [ -f .env ]; then
    source .env 2>/dev/null || true
fi

DB_PORT="${DB_PORT:-5433}"
DB_URL="postgres://et:et@localhost:${DB_PORT}/et"

echo "=== Estado Transparente - Demo Pipeline ==="
echo ""

echo "1. Copiando archivo de demo a data/raw..."
mkdir -p data/raw
DEMO_FILE="data/demo_presupuesto.csv"

if [ ! -f "$DEMO_FILE" ]; then
    echo "Error: No se encontró $DEMO_FILE"
    exit 1
fi

# Generate a UUID for the artifact (macOS compatible)
ARTIFACT_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
CAPTURED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
# macOS uses shasum instead of sha256sum
CONTENT_HASH="sha256:$(shasum -a 256 "$DEMO_FILE" | cut -d' ' -f1)"
SIZE_BYTES=$(wc -c < "$DEMO_FILE" | tr -d ' ')
STORAGE_PATH="data/raw/${ARTIFACT_ID}.raw"

cp "$DEMO_FILE" "$STORAGE_PATH"

echo "   Artifact ID: $ARTIFACT_ID"
echo "   Hash: $CONTENT_HASH"

echo ""
echo "2. Registrando artifact en base de datos..."

docker exec -i infra-db-1 psql -U et -d et <<EOF
INSERT INTO artifacts (artifact_id, source_id, url, captured_at, content_hash, mime_type, size_bytes, storage_kind, storage_path, parsed_status)
VALUES (
    '$ARTIFACT_ID',
    'demo-presupuesto',
    'file://$PROJECT_DIR/$DEMO_FILE',
    '$CAPTURED_AT',
    '$CONTENT_HASH',
    'text/csv',
    $SIZE_BYTES,
    'fs',
    '$STORAGE_PATH',
    'pending'
);
EOF

echo "   Artifact registrado"

echo ""
echo "3. Parseando artifact..."
cargo run --bin parser -- --artifact-id "$ARTIFACT_ID"

echo ""
echo "4. Verificando datos..."
echo ""
echo "Entidades:"
docker exec -i infra-db-1 psql -U et -d et -c "SELECT entity_key, display_name FROM entities LIMIT 10;"

echo ""
echo "Métricas:"
docker exec -i infra-db-1 psql -U et -d et -c "SELECT metric_key, display_name, unit FROM metrics;"

echo ""
echo "Facts (sample):"
docker exec -i infra-db-1 psql -U et -d et -c "SELECT e.display_name as entidad, m.display_name as metrica, f.period_start, f.value_num FROM facts f JOIN entities e ON f.entity_id = e.entity_id JOIN metrics m ON f.metric_id = m.metric_id LIMIT 10;"

echo ""
echo "=== Demo Pipeline Completo ==="
echo ""
echo "Ahora puedes:"
echo "  1. La API ya está corriendo en: http://localhost:8080"
echo "  2. Iniciar el Web:  cd apps/web && npm install && npm run dev"
echo "  3. Abrir:           http://localhost:5173"
echo ""
