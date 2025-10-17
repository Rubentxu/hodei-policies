#!/bin/bash
# Script para configurar el entorno de desarrollo

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🔧 Configurando entorno de desarrollo${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verificar Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust no está instalado"
    echo "Instala Rust desde: https://rustup.rs/"
    exit 1
fi

echo -e "${GREEN}✅ Rust instalado:${NC} $(rustc --version)"

# Verificar Docker
if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker no está instalado (opcional para PostgreSQL/Redis)"
else
    echo -e "${GREEN}✅ Docker instalado:${NC} $(docker --version)"
fi

# Verificar Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo "⚠️  Docker Compose no está instalado (opcional)"
else
    echo -e "${GREEN}✅ Docker Compose instalado:${NC} $(docker-compose --version)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}📦 Instalando dependencias${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Build del workspace
cargo build --workspace
echo -e "${GREEN}✅ Workspace compilado${NC}"

# Ejecutar tests
cargo test --workspace
echo -e "${GREEN}✅ Tests ejecutados${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 Entorno configurado correctamente${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Próximos pasos:"
echo "  1. Iniciar servicios: docker-compose -f docker-compose.dev.yml up -d"
echo "  2. Ejecutar ejemplo: cargo run -p app-example"
echo "  3. Ejecutar tests: make test"
