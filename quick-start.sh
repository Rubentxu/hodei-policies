#!/bin/bash

# Script de inicio rápido para Hodei Authorization Engine
set -e

echo "🚀 Hodei Authorization Engine - Quick Start"
echo "==========================================="
echo ""

# Verificar Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker no está instalado. Por favor instala Docker primero."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose no está instalado. Por favor instala Docker Compose primero."
    exit 1
fi

echo "✅ Docker y Docker Compose detectados"
echo ""

# Verificar si los servicios ya están corriendo
if docker-compose ps | grep -q "Up"; then
    echo "⚠️  Los servicios ya están corriendo. Deteniéndolos..."
    docker-compose down
    echo ""
fi

# Levantar servicios
echo "🐳 Levantando servicios con Docker Compose..."
docker-compose up -d

echo ""
echo "⏳ Esperando a que PostgreSQL esté listo..."
sleep 5

# Verificar que PostgreSQL esté listo
until docker-compose exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    echo "   Esperando PostgreSQL..."
    sleep 2
done

echo "✅ PostgreSQL está listo!"
echo ""

echo "⏳ Esperando a que la aplicación esté lista..."
sleep 10

# Verificar que la aplicación responda
MAX_RETRIES=30
RETRY_COUNT=0
until curl -s http://localhost:3000/health > /dev/null 2>&1 || [ $RETRY_COUNT -eq $MAX_RETRIES ]; do
    echo "   Esperando aplicación... ($RETRY_COUNT/$MAX_RETRIES)"
    sleep 2
    ((RETRY_COUNT++))
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ La aplicación no respondió a tiempo. Verificando logs..."
    docker-compose logs app
    exit 1
fi

echo "✅ Aplicación está lista!"
echo ""

# Ejecutar tests
echo "🧪 Ejecutando tests de API..."
echo ""
./tests/api_tests.sh

TEST_RESULT=$?

echo ""
echo "==========================================="
echo "📊 Resumen"
echo "==========================================="
echo ""

if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ ¡Todo funcionando correctamente!"
    echo ""
    echo "🌐 La aplicación está corriendo en: http://localhost:3000"
    echo ""
    echo "📚 Comandos útiles:"
    echo "   make logs-app      # Ver logs de la aplicación"
    echo "   make logs-db       # Ver logs de PostgreSQL"
    echo "   make docker-down   # Detener servicios"
    echo "   make test          # Ejecutar tests nuevamente"
    echo ""
    echo "🧪 Prueba la API:"
    echo "   curl -X POST http://localhost:3000/documents \\"
    echo "     -H 'Authorization: Bearer alice' \\"
    echo "     -H 'Content-Type: application/json' \\"
    echo "     -d '{\"resource_id\":\"test\",\"is_public\":false}'"
    echo ""
else
    echo "❌ Algunos tests fallaron. Revisa los logs:"
    echo "   make logs-app"
    echo ""
fi

exit $TEST_RESULT
