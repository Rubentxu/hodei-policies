#!/bin/bash
# Publicar crates secuencialmente en orden de dependencias
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🚀 Publicación Secuencial en crates.io${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verificar login
read -p "¿Has ejecutado 'cargo login'? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Ejecuta primero: cargo login <token>"
    exit 1
fi

# Orden de publicación (sin dependencias internas primero)
CRATES=(
    "hodei-hrn"
    "hodei-derive"
    "hodei-authz"
    "hodei-authz-postgres"
    "hodei-authz-redis"
    "hodei-authz-axum"
    "hodei-authz-sdk"
)

# Publicar cada crate
for crate in "${CRATES[@]}"; do
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}📦 Publicando: $crate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "crates/$crate"
    
    # Publicar
    if cargo publish; then
        echo -e "${GREEN}✅ $crate publicado exitosamente${NC}"
        cd ../..
        
        # Esperar para que crates.io procese
        if [ "$crate" != "hodei-authz-sdk" ]; then
            echo -e "${YELLOW}⏳ Esperando 20s para que crates.io procese...${NC}"
            sleep 20
        fi
    else
        echo -e "${RED}❌ Error publicando $crate${NC}"
        cd ../..
        exit 1
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 ¡PUBLICACIÓN COMPLETADA!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Próximos pasos:"
echo "  1. Verificar: https://crates.io/"
echo "  2. Docs: https://docs.rs/"
echo "  3. Release: git tag v0.1.0 && git push --tags"
