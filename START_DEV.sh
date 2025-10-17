#!/bin/bash
# Script rápido para iniciar el entorno de desarrollo

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 Iniciando Entorno de Desarrollo Hodei${NC}"
echo ""

# Iniciar servicios
echo "📦 Iniciando servicios (PostgreSQL + Redis)..."
docker compose -f docker-compose.dev.yml up -d

# Esperar a que estén listos
echo "⏳ Esperando a que los servicios estén listos..."
sleep 3

# Verificar PostgreSQL
if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ PostgreSQL listo${NC}"
else
    echo -e "${YELLOW}⚠️  PostgreSQL aún no está listo, esperando...${NC}"
    sleep 5
fi

# Verificar Redis
if redis-cli -h localhost -p 6379 ping > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Redis listo${NC}"
else
    echo -e "${YELLOW}⚠️  Redis aún no está listo, esperando...${NC}"
    sleep 5
fi

echo ""
echo -e "${GREEN}✅ Servicios iniciados correctamente${NC}"
echo ""
echo "📊 Servicios disponibles:"
echo "   • PostgreSQL: localhost:5432"
echo "   • Redis: localhost:6379"
echo "   • Adminer (UI): http://localhost:8080"
echo "   • pgAdmin (UI): http://localhost:5050"
echo ""
echo "🔧 Variables de entorno:"
echo "   export DATABASE_URL=\"postgres://postgres:postgres@localhost:5432/hodei_policies\""
echo "   export REDIS_URL=\"redis://localhost:6379\""
echo ""
echo "🚀 Para ejecutar la aplicación:"
echo "   cargo run -p app-example"
echo ""
echo "🧪 Para ejecutar tests:"
echo "   bash tests/app_example_tests.sh"
echo ""
echo "🛑 Para detener servicios:"
echo "   docker compose -f docker-compose.dev.yml down"
echo ""
