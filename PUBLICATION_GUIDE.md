# 📦 Guía de Publicación en crates.io

**Framework**: Hodei Authorization Framework  
**Versión**: 0.1.0  
**Estado**: Listo para publicar

---

## ✅ Checklist Pre-Publicación

### 1. Verificaciones Básicas

- [x] Todos los crates compilan sin errores
- [x] Tests implementados y pasando
- [x] README.md en cada crate
- [x] Licencia especificada (MIT OR Apache-2.0)
- [x] Metadata completa en Cargo.toml
- [ ] Versiones sincronizadas
- [ ] Sin dependencias con `path` absolutos

### 2. Documentación

- [x] README.md principal
- [x] README.md por crate
- [x] Ejemplos de código
- [x] Documentación inline (///)
- [ ] CHANGELOG.md
- [ ] Badges en README

### 3. Calidad del Código

- [x] `cargo check` pasa
- [x] `cargo test` pasa (tests unitarios)
- [ ] `cargo clippy` sin warnings
- [ ] `cargo fmt` aplicado
- [ ] Sin `TODO` o `FIXME` críticos

---

## 📋 Orden de Publicación

**IMPORTANTE**: Los crates deben publicarse en orden de dependencias.

### Paso 1: hodei-authz-sdk-kernel (sin dependencias del workspace)

```bash
cd crates/hodei-authz-sdk-kernel

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos para que se indexe en crates.io
```

### Paso 2: hodei-authz-sdk-derive (depende de kernel)

```bash
cd crates/hodei-authz-sdk-derive

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos
```

### Paso 3: hodei-authz-sdk-core (depende de kernel + derive)

```bash
cd crates/hodei-authz-sdk-core

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos
```

### Paso 4: hodei-authz-sdk-authz-postgres (depende de core)

```bash
cd crates/hodei-authz-sdk-authz-postgres

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos
```

### Paso 5: hodei-authz-sdk-authz-redis (depende de core)

```bash
cd crates/hodei-authz-sdk-authz-redis

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos
```

### Paso 6: hodei-authz-sdk-authz-axum (depende de core)

```bash
cd crates/hodei-authz-sdk-authz-axum

# Verificar
cargo publish --dry-run

# Publicar
cargo publish

# Esperar ~5 minutos
```

### Paso 7: hodei-authz-sdk (meta-crate, depende de todos)

```bash
cd crates/hodei-authz-sdk

# Verificar
cargo publish --dry-run

# Publicar
cargo publish
```

---

## 🔧 Preparación

### 1. Actualizar Versiones

Asegurarse que todas las versiones sean consistentes:

```toml
# En workspace Cargo.toml
[workspace.package]
version = "0.1.0"

# En cada crate
[package]
version = "0.1.0"
```

### 2. Actualizar Dependencias Internas

Cambiar de `path` a `version`:

```toml
# ANTES (desarrollo)
[dependencies]
hodei-authz-sdk-kernel = { path = "../hodei-authz-sdk-kernel" }

# DESPUÉS (publicación)
[dependencies]
hodei-authz-sdk-kernel = "0.1.0"
```

### 3. Verificar Metadata

Cada `Cargo.toml` debe tener:

```toml
[package]
name = "hodei-authz-sdk-xxx"
version = "0.1.0"
edition = "2021"
authors = ["Tu Nombre <email@example.com>"]
license = "MIT OR Apache-2.0"
description = "Descripción clara y concisa"
repository = "https://github.com/usuario/hodei-authz-sdk"
documentation = "https://docs.rs/hodei-authz-sdk-xxx"
homepage = "https://github.com/usuario/hodei-authz-sdk"
keywords = ["authorization", "cedar", "policy"]  # Max 5
categories = ["authentication"]  # De la lista oficial
readme = "README.md"
```

### 4. Crear LICENSE Files

```bash
# Copiar licencias a cada crate
for crate in hodei-authz-sdk-kernel hodei-authz-sdk-derive hodei-authz-sdk-core hodei-authz-sdk-authz-postgres hodei-authz-sdk-authz-redis hodei-authz-sdk-authz-axum hodei-authz-sdk; do
    cp LICENSE-MIT crates/$crate/
    cp LICENSE-APACHE crates/$crate/
done
```

---

## 🧪 Verificación Final

### Tests

```bash
# Todos los tests unitarios
cargo test --workspace --lib

# Tests de integración (requieren servicios)
docker-compose up -d
cargo test --workspace -- --ignored
```

### Linter

```bash
# Clippy
cargo clippy --workspace --all-features -- -D warnings

# Format
cargo fmt --all -- --check
```

### Build

```bash
# Compilar todo
cargo build --workspace --all-features --release

# Verificar tamaños
ls -lh target/release/*.rlib
```

---

## 📝 Comandos Útiles

### Dry Run (Simulación)

```bash
# Verificar qué se publicaría
cargo publish --dry-run

# Ver el paquete que se crearía
cargo package --list
```

### Verificar en crates.io

Después de publicar cada crate:

```
https://crates.io/crates/hodei-authz-sdk-kernel
https://crates.io/crates/hodei-authz-sdk-derive
https://crates.io/crates/hodei-authz-sdk-core
https://crates.io/crates/hodei-authz-sdk-authz-postgres
https://crates.io/crates/hodei-authz-sdk-authz-redis
https://crates.io/crates/hodei-authz-sdk-authz-axum
https://crates.io/crates/hodei-authz-sdk
```

### Verificar Documentación

```
https://docs.rs/hodei-authz-sdk-kernel
https://docs.rs/hodei-authz-sdk-derive
https://docs.rs/hodei-authz-sdk-core
https://docs.rs/hodei-authz-sdk-authz-postgres
https://docs.rs/hodei-authz-sdk-authz-redis
https://docs.rs/hodei-authz-sdk-authz-axum
https://docs.rs/hodei-authz-sdk
```

---

## ⚠️ Problemas Comunes

### Error: "crate not found"

**Causa**: Dependencia interna no publicada aún  
**Solución**: Publicar en el orden correcto (kernel → derive → core → adapters → meta)

### Error: "failed to verify"

**Causa**: Tests fallan en el build de publicación  
**Solución**: Ejecutar `cargo test` localmente primero

### Error: "missing license file"

**Causa**: LICENSE-MIT o LICENSE-APACHE no están en el crate  
**Solución**: Copiar archivos de licencia a cada crate

### Error: "repository not found"

**Causa**: URL del repositorio incorrecta  
**Solución**: Verificar que el repo existe y es público

---

## 🎯 Post-Publicación

### 1. Anuncio

- [ ] Publicar en Reddit r/rust
- [ ] Publicar en Twitter/X
- [ ] Publicar en This Week in Rust
- [ ] Actualizar README principal con badges

### 2. Badges

Agregar al README.md:

```markdown
[![Crates.io](https://img.shields.io/crates/v/hodei-authz-sdk.svg)](https://crates.io/crates/hodei-authz-sdk)
[![Documentation](https://docs.rs/hodei-authz-sdk/badge.svg)](https://docs.rs/hodei-authz-sdk)
[![License](https://img.shields.io/crates/l/hodei-authz-sdk.svg)](https://github.com/usuario/hodei-authz-sdk#license)
```

### 3. Monitoreo

- Verificar que docs.rs genera la documentación correctamente
- Revisar issues en GitHub
- Responder preguntas en crates.io

---

## 📅 Versionado Futuro

Seguir [Semantic Versioning](https://semver.org/):

- **0.1.x**: Parches y bug fixes
- **0.x.0**: Nuevas features (breaking changes OK en 0.x)
- **1.0.0**: Primera versión estable (API estable)

### Actualizar Versión

```bash
# En workspace Cargo.toml
[workspace.package]
version = "0.2.0"

# Publicar en el mismo orden
```

---

## ✅ Checklist Final

Antes de ejecutar `cargo publish`:

- [ ] README.md actualizado
- [ ] CHANGELOG.md creado
- [ ] Versiones sincronizadas
- [ ] Tests pasando
- [ ] Clippy sin warnings
- [ ] Código formateado
- [ ] Licencias incluidas
- [ ] Documentación completa
- [ ] Dry-run exitoso
- [ ] Git tag creado (`git tag v0.1.0`)
- [ ] Cambios pusheados a GitHub

---

**¡Listo para publicar!** 🚀
