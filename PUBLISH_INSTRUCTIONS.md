# 📦 Instrucciones de Publicación

## Crates a Publicar (en orden)

### 1. hodei-kernel (sin dependencias internas)
```bash
cd kernel
cargo publish --dry-run  # Verificar primero
cargo publish
cd ..
```

### 2. hodei-provider-derive (solo depende de syn/quote)
```bash
cd hodei_provider_derive
cargo publish --dry-run
cargo publish
cd ..
```

### 3. hodei-provider (depende de kernel y derive)
```bash
cd hodei_provider
cargo publish --dry-run
cargo publish
cd ..
```

## ✅ Verificación Previa

```bash
# Compilar todo
cargo build --release

# Verificar cada crate
cargo package --manifest-path kernel/Cargo.toml
cargo package --manifest-path hodei_provider_derive/Cargo.toml
cargo package --manifest-path hodei_provider/Cargo.toml
```

## 📝 Uso para Usuarios

Después de publicar, los usuarios solo necesitan:

```toml
[dependencies]
hodei-provider = "0.1.0"  # Incluye automáticamente kernel y derive
cedar-policy = "4.7.0"
```

## 🎯 Ejemplo de Uso

El código en `hodei_domain/` y `src/` sirve como ejemplo completo de cómo usar las librerías.

