#!/bin/bash

# Script de tests para Artifacts API
# Prueba el CRUD completo de artifacts con autorización Cedar

set -e

API_URL="http://localhost:3000"

# Colores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Función helper para ejecutar tests
run_test() {
    local test_name="$1"
    local expected_status="$2"
    shift 2
    
    response=$(curl -s -w "\n%{http_code}" "$@")
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)
    
    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}🧪 Test: $test_name ... ✓ PASS (HTTP $status)${NC}"
        if [ -n "$body" ]; then
            echo "   Response: $body"
        fi
    else
        echo -e "${RED}🧪 Test: $test_name ... ✗ FAIL (Expected: $expected_status, Got: $status)${NC}"
        echo "   Response: $body"
    fi
}

echo "🚀 Iniciando tests de Artifacts API"
echo "📍 API URL: $API_URL"
echo ""

# Primero crear un documento para referenciar
echo "═══════════════════════════════════════════════════════════════"
echo "📋 SETUP: Crear documento para referenciar"
echo "═══════════════════════════════════════════════════════════════"

DOC_RESPONSE=$(curl -s -X POST "$API_URL/documents" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"resource_id":"doc-for-artifacts","is_public":false}')

# Extraer el objeto HRN completo como JSON
DOC_HRN_JSON=$(echo $DOC_RESPONSE | jq -c '.id')
echo "✅ Documento creado: $DOC_HRN_JSON"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Artifact CRUD - CREATE"
echo "═══════════════════════════════════════════════════════════════"

ARTIFACT_PAYLOAD=$(jq -n \
    --arg rid "artifact-1" \
    --arg name "My First Artifact" \
    --arg type "binary" \
    --arg ver "1.0.0" \
    --argjson doc "$DOC_HRN_JSON" \
    '{resource_id: $rid, name: $name, artifact_type: $type, version: $ver, document_id: $doc}')

run_test "Alice (admin) crea artifact-1" 201 \
    -X POST "$API_URL/artifacts" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d "$ARTIFACT_PAYLOAD"

run_test "Bob (user, tenant-b) NO puede crear artifact (diferente tenant)" 403 \
    -X POST "$API_URL/artifacts" \
    -H "Authorization: Bearer bob" \
    -H "Content-Type: application/json" \
    -d '{"resource_id":"artifact-bob","name":"Bob Artifact","artifact_type":"library","version":"1.0.0"}'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Artifact CRUD - READ"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice lee su propio artifact" 200 \
    -X GET "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice"

run_test "Bob NO puede leer artifact de Alice (diferente tenant)" 404 \
    -X GET "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer bob"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Artifact CRUD - UPDATE"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice actualiza su artifact (cambiar versión)" 200 \
    -X PUT "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"version":"2.0.0","is_active":true}'

run_test "Alice actualiza su artifact (cambiar nombre)" 200 \
    -X PUT "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"name":"Updated Artifact Name"}'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Artifact CRUD - DELETE"
echo "═══════════════════════════════════════════════════════════════"

ARTIFACT2_PAYLOAD=$(jq -n \
    --arg rid "artifact-2" \
    --arg name "To Delete" \
    --arg type "temp" \
    --arg ver "1.0.0" \
    --argjson doc "$DOC_HRN_JSON" \
    '{resource_id: $rid, name: $name, artifact_type: $type, version: $ver, document_id: $doc}')

run_test "Alice crea artifact-2 para eliminar" 201 \
    -X POST "$API_URL/artifacts" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d "$ARTIFACT2_PAYLOAD"

run_test "Alice elimina artifact-2" 204 \
    -X DELETE "$API_URL/artifacts/artifact-2" \
    -H "Authorization: Bearer alice"

run_test "Verificar que artifact-2 fue eliminado" 404 \
    -X GET "$API_URL/artifacts/artifact-2" \
    -H "Authorization: Bearer alice"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Permisos de Creador (creator_permissions)"
echo "═══════════════════════════════════════════════════════════════"

run_test "Verificar que Alice puede leer artifact que creó" 200 \
    -X GET "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice"

run_test "Verificar que Alice puede actualizar artifact que creó" 200 \
    -X PUT "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"is_active":false}'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 LIMPIEZA: Eliminar artifacts de test"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice elimina artifact-1" 204 \
    -X DELETE "$API_URL/artifacts/artifact-1" \
    -H "Authorization: Bearer alice"

run_test "Alice elimina documento de referencia" 204 \
    -X DELETE "$API_URL/documents/doc-for-artifacts" \
    -H "Authorization: Bearer alice"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 RESUMEN DE TESTS DE ARTIFACTS"
echo "═══════════════════════════════════════════════════════════════"
echo "✅ Tests completados"
echo ""
