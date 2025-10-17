#!/bin/bash

# Script de tests de API para validar el motor de autorización Hodei
# Colores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_URL="${API_URL:-http://localhost:3000}"
FAILED_TESTS=0
PASSED_TESTS=0

echo "🚀 Iniciando tests de API para Hodei Authorization Engine"
echo "📍 API URL: $API_URL"
echo ""

# Función para ejecutar test
run_test() {
    local test_name="$1"
    local expected_status="$2"
    shift 2
    local curl_args=("$@")
    
    echo -n "🧪 Test: $test_name ... "
    
    response=$(curl -s -w "\n%{http_code}" "${curl_args[@]}")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code)"
        ((PASSED_TESTS++))
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "   Response: $(echo $body | jq -c . 2>/dev/null || echo $body)"
        fi
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Expected: $expected_status, Got: $http_code)"
        ((FAILED_TESTS++))
        if [ -n "$body" ]; then
            echo "   Response: $body"
        fi
        return 1
    fi
}

echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: REQ-HRN-04 - Hidratación de HRN en Backend"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice (admin, tenant-a) crea documento doc-test1" 200 \
    -X POST "$API_URL/documents" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"resource_id":"doc-test1","is_public":false}'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: REQ-PM-01 - Aislamiento Multi-Tenant"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice (tenant-a) lee su propio documento" 200 \
    -X GET "$API_URL/documents/doc-test1" \
    -H "Authorization: Bearer alice"

run_test "Bob (tenant-b) NO puede leer documento de Alice (tenant-a)" 403 \
    -X GET "$API_URL/documents/doc-test1" \
    -H "Authorization: Bearer bob"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: REQ-SVC-05 - Flujo de Autorización Multi-Tenant"
echo "═══════════════════════════════════════════════════════════════"

run_test "Bob (user, tenant-b) crea documento doc-test2" 200 \
    -X POST "$API_URL/documents" \
    -H "Authorization: Bearer bob" \
    -H "Content-Type: application/json" \
    -d '{"resource_id":"doc-test2","is_public":true}'

run_test "Bob lee su propio documento" 200 \
    -X GET "$API_URL/documents/doc-test2" \
    -H "Authorization: Bearer bob"

run_test "Alice NO puede leer documento de Bob (diferente tenant)" 403 \
    -X GET "$API_URL/documents/doc-test2" \
    -H "Authorization: Bearer alice"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Permisos de Propietario (Owner Permissions)"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice actualiza su propio documento" 200 \
    -X PUT "$API_URL/documents/doc-test1" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"is_public":true}'

run_test "Bob NO puede actualizar documento de Alice" 403 \
    -X PUT "$API_URL/documents/doc-test1" \
    -H "Authorization: Bearer bob" \
    -H "Content-Type: application/json" \
    -d '{"is_public":false}'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Permisos de Eliminación"
echo "═══════════════════════════════════════════════════════════════"

run_test "Bob elimina su propio documento" 204 \
    -X DELETE "$API_URL/documents/doc-test2" \
    -H "Authorization: Bearer bob"

run_test "Verificar que documento de Bob fue eliminado" 404 \
    -X GET "$API_URL/documents/doc-test2" \
    -H "Authorization: Bearer bob"

run_test "Alice NO puede eliminar documento que no existe" 404 \
    -X DELETE "$API_URL/documents/doc-test2" \
    -H "Authorization: Bearer alice"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: REQ-API-01 - Gestión Dinámica de Políticas"
echo "═══════════════════════════════════════════════════════════════"

# Crear una política temporal que permita lectura pública
PUBLIC_READ_POLICY='permit(principal, action == Action::"Read", resource) when { resource.is_public == true };'

run_test "Agregar política de lectura pública" 201 \
    -X POST "$API_URL/_api/policies/public_read" \
    -H "Content-Type: text/plain" \
    -d "$PUBLIC_READ_POLICY"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: REQ-DM-01 - Recursos Virtuales (Creación)"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice crea documento público doc-test3" 200 \
    -X POST "$API_URL/documents" \
    -H "Authorization: Bearer alice" \
    -H "Content-Type: application/json" \
    -d '{"resource_id":"doc-test3","is_public":true}'

# Nota: Con la política pública, Bob aún no puede leer porque está en diferente tenant
run_test "Bob NO puede leer doc público de Alice (diferente tenant)" 403 \
    -X GET "$API_URL/documents/doc-test3" \
    -H "Authorization: Bearer bob"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 REQUISITO: Limpieza - Eliminar documentos de test"
echo "═══════════════════════════════════════════════════════════════"

run_test "Alice elimina doc-test1" 204 \
    -X DELETE "$API_URL/documents/doc-test1" \
    -H "Authorization: Bearer alice"

run_test "Alice elimina doc-test3" 204 \
    -X DELETE "$API_URL/documents/doc-test3" \
    -H "Authorization: Bearer alice"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 RESUMEN DE TESTS"
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}✓ Tests Pasados: $PASSED_TESTS${NC}"
echo -e "${RED}✗ Tests Fallidos: $FAILED_TESTS${NC}"
echo "Total: $((PASSED_TESTS + FAILED_TESTS))"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 ¡Todos los tests pasaron exitosamente!${NC}"
    echo ""
    echo "✅ Requisitos Validados:"
    echo "   • REQ-HRN-04: Hidratación de HRN en backend"
    echo "   • REQ-PM-01: Aislamiento multi-tenant estricto"
    echo "   • REQ-SVC-05: Flujo de autorización completo"
    echo "   • REQ-API-01: Gestión dinámica de políticas"
    echo "   • REQ-DM-01: Recursos virtuales en creación"
    exit 0
else
    echo -e "${RED}❌ Algunos tests fallaron. Revisa los errores arriba.${NC}"
    exit 1
fi
