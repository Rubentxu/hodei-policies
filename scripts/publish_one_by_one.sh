#!/bin/bash
# Publicar crates uno por uno, cambiando dependencias solo cuando sea necesario
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🚀 Publicación en crates.io (uno por uno)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verificar login
read -p "¿Has ejecutado 'cargo login'? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Ejecuta primero: cargo login <token>"
    exit 1
fi

VERSION="0.1.0"

# Función para publicar un crate
publish_crate() {
    local crate=$1
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}📦 Publicando: $crate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "crates/$crate"
    
    # Crear backup
    cp Cargo.toml Cargo.toml.backup
    
    # Cambiar workspace a versiones (solo para este crate)
    sed -i 's/hodei-hrn = { workspace = true }/hodei-hrn = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-derive = { workspace = true }/hodei-derive = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz = { workspace = true }/hodei-authz = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-postgres = { workspace = true, optional = true }/hodei-authz-postgres = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-postgres = { workspace = true }/hodei-authz-postgres = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-redis = { workspace = true, optional = true }/hodei-authz-redis = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-redis = { workspace = true }/hodei-authz-redis = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-axum = { workspace = true, optional = true }/hodei-authz-axum = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-axum = { workspace = true }/hodei-authz-axum = "'$VERSION'"/g' Cargo.toml
    
    # Publicar (sin verificar workspace completo)
    if cargo publish --no-verify; then
        echo -e "${GREEN}✅ $crate publicado exitosamente${NC}"
        
        # Restaurar
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        
        # Esperar
        if [ "$crate" != "hodei-authz-sdk" ]; then
            echo -e "${YELLOW}⏳ Esperando 20s para que crates.io procese...${NC}"
            sleep 20
        fi
        
        return 0
    else
        echo -e "${RED}❌ Error publicando $crate${NC}"
        # Restaurar en caso de error
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        return 1
    fi
}

# Publicar en orden
publish_crate "hodei-hrn" || exit 1
publish_crate "hodei-derive" || exit 1
publish_crate "hodei-authz" || exit 1
publish_crate "hodei-authz-postgres" || exit 1
publish_crate "hodei-authz-redis" || exit 1
publish_crate "hodei-authz-axum" || exit 1
publish_crate "hodei-authz-sdk" || exit 1

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 ¡PUBLICACIÓN COMPLETADA!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Próximos pasos:"
echo "  1. Verificar: https://crates.io/"
echo "  2. Docs: https://docs.rs/"
echo "  3. Release: git tag v0.1.0 && git push --tags"
echo "  4. Actualizar README con badges"
