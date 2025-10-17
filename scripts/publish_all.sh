#!/bin/bash
# Script completo: preparar, publicar y restaurar

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🚀 Publicación Completa de Hodei en crates.io${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verificar que estamos en la raíz del proyecto
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Ejecuta este script desde la raíz del proyecto${NC}"
    exit 1
fi

# Verificar que cargo login está configurado
echo "🔐 Verificando autenticación..."
if ! cargo login --help > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: cargo no está instalado${NC}"
    exit 1
fi

echo ""
read -p "¿Has ejecutado 'cargo login' con tu API token? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Por favor ejecuta primero:"
    echo "  cargo login <tu-api-token>"
    echo ""
    echo "Obtén tu token en: https://crates.io/me"
    exit 0
fi

# Paso 1: Preparar para publicación
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 PASO 1: Preparando crates"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./prepare_publish.sh

# Paso 2: Verificar compilación
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 PASO 2: Verificando compilación"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if ! cargo check --workspace; then
    echo -e "${RED}❌ Error: La compilación falló${NC}"
    echo "Restaurando workspace..."
    ./restore_workspace.sh
    exit 1
fi

# Paso 3: Publicar
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 PASO 3: Publicando crates"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Confirmar publicación
echo ""
echo -e "${YELLOW}⚠️  Estás a punto de publicar en crates.io${NC}"
echo ""
read -p "¿Continuar con la publicación? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Publicación cancelada. Restaurando workspace..."
    ./restore_workspace.sh
    exit 0
fi

# Ejecutar script de publicación
./publish.sh

PUBLISH_STATUS=$?

# Paso 4: Restaurar workspace
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔄 PASO 4: Restaurando workspace"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./restore_workspace.sh

if [ $PUBLISH_STATUS -eq 0 ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}🎉 ¡PUBLICACIÓN COMPLETADA!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📋 Próximos pasos:"
    echo "  1. Verificar en https://crates.io/users/tu-usuario"
    echo "  2. Verificar docs en https://docs.rs/"
    echo "  3. Crear release: git tag v0.1.0 && git push --tags"
    echo "  4. Actualizar README con badges"
    echo ""
else
    echo ""
    echo -e "${RED}❌ La publicación falló${NC}"
    echo "Revisa los errores arriba"
    exit 1
fi
