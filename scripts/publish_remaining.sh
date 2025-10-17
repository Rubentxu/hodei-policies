#!/bin/bash
# Publicar los 2 crates restantes del SDK
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}📦 Publicando crates restantes del SDK${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

VERSION="0.1.0"

# Backup del Cargo.toml raíz
cp Cargo.toml Cargo.toml.backup 2>/dev/null || true

# Función para publicar un crate
publish_crate() {
    local crate=$1
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}📦 Publicando: $crate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Modificar workspace para incluir solo este crate
    cat > Cargo.toml << 'EOF'
[workspace]
members = ["crates/CRATE_NAME"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"

[workspace.dependencies]
EOF
    sed -i "s/CRATE_NAME/$crate/g" Cargo.toml
    
    # Agregar dependencias del workspace original
    grep -A 100 "^\[workspace.dependencies\]" Cargo.toml.backup | tail -n +2 >> Cargo.toml
    
    cd "crates/$crate"
    cp Cargo.toml Cargo.toml.backup
    
    # Cambiar workspace a versiones
    sed -i 's/hodei-hrn = { workspace = true }/hodei-hrn = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-derive = { workspace = true }/hodei-derive = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz = { workspace = true }/hodei-authz = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-postgres = { workspace = true, optional = true }/hodei-authz-postgres = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-redis = { workspace = true, optional = true }/hodei-authz-redis = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-axum = { workspace = true, optional = true }/hodei-authz-axum = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    
    # Publicar
    if cargo publish --allow-dirty; then
        echo -e "${GREEN}✅ $crate publicado${NC}"
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Error${NC}"
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        return 1
    fi
}

# Publicar hodei-authz-axum
publish_crate "hodei-authz-axum" || { mv Cargo.toml.backup Cargo.toml 2>/dev/null; exit 1; }

echo -e "${YELLOW}⏳ Esperando 20s antes de publicar el siguiente...${NC}"
sleep 20

# Publicar hodei-authz-sdk
publish_crate "hodei-authz-sdk" || { mv Cargo.toml.backup Cargo.toml 2>/dev/null; exit 1; }

# Restaurar workspace original
mv Cargo.toml.backup Cargo.toml 2>/dev/null || git checkout Cargo.toml

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 ¡PUBLICACIÓN COMPLETADA!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Crates publicados en crates.io:"
echo "  1. hodei-hrn"
echo "  2. hodei-derive"
echo "  3. hodei-authz"
echo "  4. hodei-authz-postgres"
echo "  5. hodei-authz-redis"
echo "  6. hodei-authz-axum"
echo "  7. hodei-authz-sdk"
echo ""
echo "📋 Próximos pasos:"
echo "  1. Verificar: https://crates.io/search?q=hodei"
echo "  2. Docs: https://docs.rs/hodei-authz-sdk"
echo "  3. Release: git tag v0.1.0 && git push --tags"
echo "  4. Actualizar README con badges"
