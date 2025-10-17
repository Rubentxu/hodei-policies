# 🔢 Versionado Centralizado con Workspace

## ❓ Pregunta

**¿Se puede centralizar el versionado en el `Cargo.toml` raíz para publicar en crates.io?**

**Respuesta**: Sí, pero con limitaciones importantes.

## ✅ Ventajas del Versionado Centralizado

```toml
# Cargo.toml (raíz)
[workspace.package]
version = "0.1.0"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
edition = "2021"

[workspace]
members = [
    "crates/hodei-hrn",
    "crates/hodei-derive",
    # ...
]
```

```toml
# crates/hodei-hrn/Cargo.toml
[package]
name = "hodei-hrn"
version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
edition.workspace = true
```

### Beneficios
1. **DRY** - Una sola fuente de verdad
2. **Consistencia** - Todos los crates con la misma metadata
3. **Mantenimiento** - Cambiar versión en un solo lugar
4. **Sincronización** - Todos los crates se versionan juntos

## ⚠️ Limitaciones para Publicación

### 1. **Versiones Independientes NO son Posibles**

```toml
# ❌ NO puedes hacer esto con workspace inheritance:
hodei-hrn = "0.1.0"
hodei-derive = "0.2.0"  # Versión diferente
hodei-authz = "0.1.5"   # Otra versión
```

**Todos los crates tendrán la MISMA versión.**

### 2. **Publicación Sincronizada Obligatoria**

Si cambias la versión workspace a `0.2.0`:
- **TODOS** los crates deben republicarse
- Aunque solo hayas cambiado uno
- Incrementa el número de versiones en crates.io

### 3. **Dependencias Internas Complejas**

```toml
# crates/hodei-authz/Cargo.toml
[dependencies]
# ❌ NO funciona con workspace version:
hodei-hrn.workspace = true

# ✅ Debes especificar versión explícita:
hodei-hrn = "0.1.0"

# O usar path para desarrollo:
hodei-hrn = { version = "0.1.0", path = "../hodei-hrn" }
```

## 🎯 Estrategias Recomendadas

### Opción 1: Versionado Independiente (RECOMENDADO para crates.io)

```toml
# crates/hodei-hrn/Cargo.toml
[package]
name = "hodei-hrn"
version = "0.1.0"  # Versión independiente

# crates/hodei-derive/Cargo.toml
[package]
name = "hodei-derive"
version = "0.2.0"  # Puede ser diferente

# crates/hodei-authz/Cargo.toml
[package]
name = "hodei-authz"
version = "0.1.5"  # Otra versión
```

**Ventajas**:
- ✅ Cada crate evoluciona a su ritmo
- ✅ Solo republicas lo que cambió
- ✅ Usuarios pueden elegir versiones específicas
- ✅ Menos ruido en crates.io

**Desventajas**:
- ❌ Más mantenimiento manual
- ❌ Posibles incompatibilidades entre versiones

### Opción 2: Versionado Sincronizado (Para monorepos internos)

```toml
# Cargo.toml (raíz)
[workspace.package]
version = "0.1.0"  # Todos juntos

# Todos los crates
[package]
version.workspace = true
```

**Ventajas**:
- ✅ Simplicidad
- ✅ Garantía de compatibilidad
- ✅ Fácil de mantener

**Desventajas**:
- ❌ Todos se versionan juntos
- ❌ Muchas versiones en crates.io
- ❌ Usuarios descargan actualizaciones innecesarias

### Opción 3: Híbrido (MEJOR PRÁCTICA)

```toml
# Cargo.toml (raíz) - Metadata compartida
[workspace.package]
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
edition = "2021"
# NO incluir version aquí

# Cada crate tiene su versión
[package]
name = "hodei-hrn"
version = "0.1.0"  # Independiente
authors.workspace = true
license.workspace = true
repository.workspace = true
edition.workspace = true
```

**Ventajas**:
- ✅ Metadata centralizada (DRY)
- ✅ Versiones independientes
- ✅ Flexibilidad máxima
- ✅ Mejor para crates.io

## 📊 Comparación

| Aspecto | Independiente | Sincronizado | Híbrido |
|---------|--------------|--------------|---------|
| Mantenimiento | Manual | Automático | Semi-automático |
| Flexibilidad | Alta | Baja | Alta |
| Publicaciones | Solo necesarias | Todas juntas | Solo necesarias |
| Compatibilidad | Manual | Garantizada | Manual |
| Recomendado para | crates.io | Interno | **crates.io** ✅ |

## 🚀 Implementación Recomendada para Hodei

```toml
# Cargo.toml (raíz)
[workspace]
members = [
    "crates/hodei-hrn",
    "crates/hodei-derive",
    "crates/hodei-authz",
    "crates/hodei-authz-postgres",
    "crates/hodei-authz-redis",
    "crates/hodei-authz-axum",
    "crates/hodei-authz-sdk",
]

[workspace.package]
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
homepage = "https://github.com/Rubentxu/hodei-policies"
edition = "2021"
# NO version aquí - cada crate la define

[workspace.dependencies]
# Dependencias externas compartidas
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
# ...
```

```toml
# crates/hodei-hrn/Cargo.toml
[package]
name = "hodei-hrn"
version = "0.1.0"  # ⭐ Versión independiente
authors.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
edition.workspace = true
description = "Hodei Resource Names"
keywords = ["hrn", "resource", "identifier"]

[dependencies]
serde.workspace = true
```

```toml
# crates/hodei-authz/Cargo.toml
[package]
name = "hodei-authz"
version = "0.1.0"  # ⭐ Puede ser diferente
authors.workspace = true
license.workspace = true
# ... metadata compartida

[dependencies]
hodei-hrn = "0.1.0"  # ⭐ Versión específica
hodei-derive = "0.1.0"
serde.workspace = true
```

## 🔄 Workflow de Actualización

### Cambio en hodei-hrn (0.1.0 → 0.1.1)

```bash
# 1. Actualizar versión en hodei-hrn/Cargo.toml
version = "0.1.1"

# 2. Publicar solo hodei-hrn
cd crates/hodei-hrn
cargo publish

# 3. Actualizar dependientes (opcional)
# Solo si quieren usar la nueva versión
# crates/hodei-authz/Cargo.toml
hodei-hrn = "0.1.1"  # Actualizar si es necesario
```

### Cambio Mayor (0.1.x → 0.2.0)

```bash
# 1. Actualizar hodei-hrn
version = "0.2.0"

# 2. Publicar
cargo publish

# 3. Actualizar TODOS los dependientes
# Porque es breaking change
hodei-hrn = "0.2.0"

# 4. Republicar dependientes
```

## 📝 Script para Sincronizar Versiones

```bash
#!/bin/bash
# sync_versions.sh - Actualizar versiones de dependencias internas

VERSION="0.1.0"

# Actualizar todas las referencias a hodei-hrn
find crates -name "Cargo.toml" -exec sed -i \
    "s/hodei-hrn = \"[0-9.]*\"/hodei-hrn = \"$VERSION\"/g" {} \;

# Actualizar hodei-derive
find crates -name "Cargo.toml" -exec sed -i \
    "s/hodei-derive = \"[0-9.]*\"/hodei-derive = \"$VERSION\"/g" {} \;

# Etc...
```

## 🎯 Recomendación Final

**Para Hodei, usa el enfoque HÍBRIDO**:

1. ✅ **Metadata centralizada** (authors, license, repo)
2. ✅ **Versiones independientes** por crate
3. ✅ **Dependencias con versiones específicas**
4. ✅ **Workspace dependencies** para externos

Esto te da:
- Flexibilidad para versionar cada crate
- Mantenimiento simplificado de metadata
- Mejor experiencia en crates.io
- Control fino sobre publicaciones

## 📚 Referencias

- [Cargo Workspace Inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table)
- [Publishing Best Practices](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [SemVer](https://semver.org/)
