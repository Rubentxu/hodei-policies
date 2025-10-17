# 📦 Guía para Publicar en crates.io

## 🔑 Paso 1: Autenticación

```bash
# 1. Ve a https://crates.io/me y genera un token API
# 2. Ejecuta:
cargo login <tu-token>
```

## 📝 Paso 2: Actualizar Metadata de Crates

Cada crate necesita esta metadata en su `Cargo.toml`:

### kernel/Cargo.toml
```toml
[package]
name = "hodei-kernel"  # Cambiar nombre (kernel está tomado)
version = "0.1.0"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
description = "Core types and traits for Hodei authorization system"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
homepage = "https://github.com/Rubentxu/hodei-policies"
documentation = "https://docs.rs/hodei-kernel"
keywords = ["authorization", "cedar", "policy", "security", "multi-tenant"]
categories = ["authentication", "web-programming"]
readme = "../README.md"
```

### hodei_provider/Cargo.toml
```toml
[package]
name = "hodei-provider"
version = "0.1.0"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
description = "Provider traits and inventory system for Hodei authorization"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
homepage = "https://github.com/Rubentxu/hodei-policies"
documentation = "https://docs.rs/hodei-provider"
keywords = ["authorization", "cedar", "policy", "security"]
categories = ["authentication", "web-programming"]
readme = "../README.md"
```

### hodei_provider_derive/Cargo.toml
```toml
[package]
name = "hodei-provider-derive"
version = "0.1.0"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
description = "Procedural macros for Hodei authorization system"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
homepage = "https://github.com/Rubentxu/hodei-policies"
documentation = "https://docs.rs/hodei-provider-derive"
keywords = ["authorization", "cedar", "macros", "derive"]
categories = ["authentication", "development-tools::procedural-macro-helpers"]
readme = "../README.md"
```

### hodei_domain/Cargo.toml
```toml
[package]
name = "hodei-domain"
version = "0.1.0"
edition = "2024"
authors = ["Ruben Dario Cabrera Garcia <rubentxu74@gmail.com>"]
description = "Domain entities and commands for Hodei authorization"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Rubentxu/hodei-policies"
homepage = "https://github.com/Rubentxu/hodei-policies"
documentation = "https://docs.rs/hodei-domain"
keywords = ["authorization", "cedar", "domain", "entities"]
categories = ["authentication", "web-programming"]
readme = "../README.md"
```

## 📄 Paso 3: Agregar Licencia

Elige una licencia (recomiendo MIT o Apache-2.0):

```bash
# Crear archivo LICENSE-MIT
cat > LICENSE-MIT << 'EOF'
MIT License

Copyright (c) 2025 Ruben Dario Cabrera Garcia

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
```

## 🔍 Paso 4: Verificar antes de publicar

```bash
# Verificar cada crate
cargo package --manifest-path kernel/Cargo.toml --allow-dirty
cargo package --manifest-path hodei_provider/Cargo.toml --allow-dirty
cargo package --manifest-path hodei_provider_derive/Cargo.toml --allow-dirty
cargo package --manifest-path hodei_domain/Cargo.toml --allow-dirty
```

## 🚀 Paso 5: Publicar (en orden)

**IMPORTANTE**: Publicar en este orden debido a dependencias:

```bash
# 1. Primero kernel (no tiene dependencias internas)
cd kernel
cargo publish

# 2. Luego hodei_provider (depende de kernel)
cd ../hodei_provider
cargo publish

# 3. Después hodei_provider_derive (depende de hodei_provider)
cd ../hodei_provider_derive
cargo publish

# 4. Finalmente hodei_domain (depende de todos)
cd ../hodei_domain
cargo publish
```

## ⚠️ Notas Importantes

1. **Nombres únicos**: Los nombres con guiones bajos (`_`) se convierten a guiones (`-`) en crates.io
2. **Versiones**: Empieza con `0.1.0` para indicar que es alpha/beta
3. **No se puede borrar**: Una vez publicado, no puedes eliminar una versión
4. **Yank**: Puedes "yankar" versiones con problemas: `cargo yank --vers 0.1.0`
5. **Actualizar dependencias**: Después de publicar cada crate, actualiza las dependencias en los siguientes

## 🔄 Actualizar Dependencias Después de Publicar

Una vez publicados, actualiza los `Cargo.toml` para usar las versiones de crates.io:

```toml
[dependencies]
hodei-kernel = "0.1.0"
hodei-provider = "0.1.0"
hodei-provider-derive = "0.1.0"
hodei-domain = "0.1.0"
```

## 📊 Después de Publicar

1. Verifica en https://crates.io/crates/hodei-kernel
2. La documentación se generará automáticamente en https://docs.rs
3. Agrega badges al README:

```markdown
[![Crates.io](https://img.shields.io/crates/v/hodei-kernel.svg)](https://crates.io/crates/hodei-kernel)
[![Documentation](https://docs.rs/hodei-kernel/badge.svg)](https://docs.rs/hodei-kernel)
[![License](https://img.shields.io/crates/l/hodei-kernel.svg)](https://github.com/Rubentxu/hodei-policies#license)
```

## 🎯 Checklist Rápido

- [ ] Obtener token de crates.io
- [ ] `cargo login`
- [ ] Actualizar metadata en todos los Cargo.toml
- [ ] Agregar archivo LICENSE
- [ ] Verificar con `cargo package` cada crate
- [ ] Publicar en orden: kernel → provider → provider_derive → domain
- [ ] Verificar en crates.io
- [ ] Actualizar README con badges
