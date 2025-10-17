#!/bin/bash
# Script para publicar Hodei en crates.io
set -e

echo "🚀 Publishing Hodei to crates.io"
echo ""

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Array de crates en orden de dependencias
CRATES=(
    "hodei-hrn"
    "hodei-derive"
    "hodei-authz"
    "hodei-authz-postgres"
    "hodei-authz-redis"
    "hodei-authz-axum"
    "hodei-authz-sdk"
)

# Verificar que estamos en el directorio correcto
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: No se encontró Cargo.toml${NC}"
    echo "Ejecuta este script desde la raíz del proyecto"
    exit 1
fi

# Verificar que cargo login está configurado
if ! cargo login --help > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: cargo no está instalado${NC}"
    exit 1
fi

echo -e "${YELLOW}⚠️  IMPORTANTE:${NC}"
echo "1. Asegúrate de haber ejecutado 'cargo login' con tu API token"
echo "2. Verifica que todas las dependencias workspace estén actualizadas a versiones específicas"
echo "3. Este script hará dry-run primero y pedirá confirmación"
echo ""
read -p "¿Continuar? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelado"
    exit 0
fi

for crate in "${CRATES[@]}"; do
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}📦 Procesando: $crate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "crates/$crate"
    
    # Verificar que compila
    echo "🔨 Compilando..."
    if ! cargo build --release; then
        echo -e "${RED}❌ Error compilando $crate${NC}"
        exit 1
    fi
    
    # Ejecutar tests
    echo "🧪 Ejecutando tests..."
    if ! cargo test; then
        echo -e "${RED}❌ Tests fallaron en $crate${NC}"
        exit 1
    fi
    
    # Dry run
    echo "🔍 Dry-run de publicación..."
    if ! cargo publish --dry-run; then
        echo -e "${RED}❌ Dry-run falló para $crate${NC}"
        echo "Revisa los errores arriba"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Dry-run exitoso${NC}"
    echo ""
    
    # Preguntar confirmación
    read -p "¿Publicar $crate en crates.io? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📤 Publicando..."
        if cargo publish; then
            echo -e "${GREEN}✅ $crate publicado exitosamente${NC}"
            # Esperar para que crates.io procese
            echo "⏳ Esperando 15 segundos para que crates.io procese..."
            sleep 15
        else
            echo -e "${RED}❌ Error publicando $crate${NC}"
            exit 1
        fi
    else
        echo -e "${YELLOW}⏭️  Saltando $crate${NC}"
        read -p "¿Continuar con el siguiente crate? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Publicación cancelada"
            exit 0
        fi
    fi
    
    cd ../..
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}🎉 ¡Publicación completada!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Próximos pasos:"
echo "1. Verificar en https://crates.io/users/tu-usuario"
echo "2. Verificar docs en https://docs.rs/"
echo "3. Crear release en GitHub: git tag v0.1.0 && git push --tags"
echo "4. Actualizar README con badges e instrucciones de instalación"
echo ""
