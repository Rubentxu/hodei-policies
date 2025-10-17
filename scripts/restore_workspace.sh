#!/bin/bash
# Script para restaurar dependencias workspace después de publicar

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔄 Restaurando dependencias workspace...${NC}"
echo ""

# Opción 1: Restaurar desde backup
if ls crates/*/Cargo.toml.backup 1> /dev/null 2>&1; then
    echo "📦 Restaurando desde backups..."
    find crates -name "Cargo.toml.backup" | while read backup; do
        original="${backup%.backup}"
        cp "$backup" "$original"
        rm "$backup"
        echo "  ✓ Restaurado: $original"
    done
else
    # Opción 2: Revertir con git
    echo "📦 Restaurando con git..."
    git checkout crates/*/Cargo.toml
fi

echo ""
echo -e "${GREEN}✅ Dependencias workspace restauradas${NC}"
echo ""
echo "Verificar cambios:"
echo "  git status"
