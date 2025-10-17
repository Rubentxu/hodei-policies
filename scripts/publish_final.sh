#!/bin/bash
# Publicar crates modificando temporalmente el workspace
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}🚀 Publicación en crates.io${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Backup del Cargo.toml raíz
cp Cargo.toml Cargo.toml.backup

VERSION="0.1.0"

# Función para publicar un crate
publish_crate() {
    local crate=$1
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}📦 Publicando: $crate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Modificar workspace para incluir solo este crate
    cat > Cargo.toml << EOF
[workspace]
members = ["crates/$crate"]
resolver = "2"

[workspace.package]
version = "$VERSION"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"

[workspace.dependencies]
$(grep -A 100 "^\[workspace.dependencies\]" Cargo.toml.backup | tail -n +2)
EOF
    
    cd "crates/$crate"
    cp Cargo.toml Cargo.toml.backup
    
    # Cambiar workspace a versiones
    sed -i 's/hodei-hrn = { workspace = true }/hodei-hrn = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-derive = { workspace = true }/hodei-derive = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz = { workspace = true }/hodei-authz = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-postgres = { workspace = true, optional = true }/hodei-authz-postgres = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-postgres = { workspace = true }/hodei-authz-postgres = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-redis = { workspace = true, optional = true }/hodei-authz-redis = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-redis = { workspace = true }/hodei-authz-redis = "'$VERSION'"/g' Cargo.toml
    sed -i 's/hodei-authz-axum = { workspace = true, optional = true }/hodei-authz-axum = { version = "'$VERSION'", optional = true }/g' Cargo.toml
    sed -i 's/hodei-authz-axum = { workspace = true }/hodei-authz-axum = "'$VERSION'"/g' Cargo.toml
    
    # Publicar
    if cargo publish --allow-dirty; then
        echo -e "${GREEN}✅ $crate publicado${NC}"
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        
        if [ "$crate" != "hodei-authz-sdk" ]; then
            echo -e "${YELLOW}⏳ Esperando 20s...${NC}"
            sleep 20
        fi
        return 0
    else
        echo -e "${RED}❌ Error${NC}"
        mv Cargo.toml.backup Cargo.toml
        cd ../..
        return 1
    fi
}

# Publicar
publish_crate "hodei-hrn" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-derive" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-authz" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-authz-postgres" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-authz-redis" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-authz-axum" || { mv Cargo.toml.backup Cargo.toml; exit 1; }
publish_crate "hodei-authz-sdk" || { mv Cargo.toml.backup Cargo.toml; exit 1; }

# Restaurar workspace original
mv Cargo.toml.backup Cargo.toml

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 ¡PUBLICACIÓN COMPLETADA!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
