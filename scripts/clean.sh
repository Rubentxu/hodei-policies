#!/bin/bash
# Script para limpiar archivos temporales y builds

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🧹 Limpiando proyecto${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Limpiar builds
echo "🗑️  Limpiando target/"
cargo clean

# Limpiar backups
echo "🗑️  Limpiando backups"
find . -name "*.backup" -delete

# Limpiar logs
echo "🗑️  Limpiando logs"
find . -name "*.log" -delete

# Limpiar archivos temporales
echo "🗑️  Limpiando archivos temporales"
rm -f /tmp/dry-run.log
rm -f /tmp/publish-*.log
rm -f /tmp/fix_*.sh

echo ""
echo -e "${GREEN}✅ Proyecto limpiado${NC}"
