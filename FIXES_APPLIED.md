# Correcciones Aplicadas - Motor de Autorización Hodei

## Problema 1: Esquema Cedar Vacío

### Causa Raíz
El crate `hodei_domain` no se estaba enlazando en el build script porque no se usaba explícitamente. El sistema de `inventory` de Rust depende de la "magia del linker" - si un crate no se referencia, el linker no lo incluye en el binario final, y por lo tanto `inventory::iter` no encuentra ningún item registrado.

### Solución Implementada
```rust
// En build.rs
#[allow(unused_imports)]
use hodei_domain as _;
```

Esta línea fuerza al linker a incluir el crate `hodei_domain`, permitiendo que `inventory` recolecte los fragmentos de esquema registrados por los macros `#[derive(HodeiEntity)]` y `#[derive(HodeiAction)]`.

**Referencia**: [Rust Forum - Help with inventory crate](https://users.rust-lang.org/t/help-with-inventory-crate/25411/3)

---

## Problema 2: Errores de Compilación de SQLx

### Causa Raíz
Los macros `query!` y `query_as!` de SQLx requieren `DATABASE_URL` en tiempo de compilación para verificar las consultas SQL contra el esquema de la base de datos. Sin esta variable, la compilación falla.

### Soluciones Aplicadas

#### Opción 1: Usar Queries Runtime (Implementada)
Reemplazamos todos los macros de compilación con sus equivalentes runtime:

**Antes**:
```rust
sqlx::query!("SELECT content FROM policies")
    .fetch_all(&self.db).await?
```

**Después**:
```rust
sqlx::query("SELECT content FROM policies")
    .fetch_all(&self.db).await?
```

**Ventajas**:
- ✅ No requiere `DATABASE_URL` en compilación
- ✅ Compilación más rápida
- ✅ Funciona en CI/CD sin base de datos

**Desventajas**:
- ⚠️ Sin verificación en tiempo de compilación
- ⚠️ Errores SQL se detectan en runtime

#### Opción 2: Modo Offline de SQLx (Alternativa)
Para proyectos que quieran verificación en tiempo de compilación:

1. **Configurar `.env`**:
   ```bash
   DATABASE_URL=postgresql://postgres:password@localhost:5432/hodei_policies
   ```

2. **Generar metadata offline**:
   ```bash
   cargo sqlx prepare
   ```

3. **Configurar en `Cargo.toml`**:
   ```toml
   [package.metadata.sqlx]
   offline = true
   ```

4. **Compilar sin base de datos**:
   ```bash
   export SQLX_OFFLINE=true
   cargo build
   ```

**Referencia**: [SQLx Documentation - Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

---

## Cambios Realizados en el Código

### 1. `build.rs`
- ✅ Agregado `use hodei_domain as _;` para forzar enlace
- ✅ Comentarios explicativos sobre el problema de inventory

### 2. `src/auth.rs`
- ✅ Reemplazado `query!` → `query`
- ✅ Agregado `use sqlx::Row` para `try_get`
- ✅ Uso de `.bind()` para parámetros

### 3. `src/main.rs`
- ✅ Reemplazado `query_as!` → `query_as::<_, Type>`
- ✅ Reemplazado `query!` → `query`
- ✅ Uso de `.bind()` para todos los parámetros
- ✅ Eliminado casts `as Hrn` innecesarios

### 4. `Cargo.toml`
- ✅ Agregado `default-features = false` para sqlx
- ✅ Configurado `[package.metadata.sqlx]` para modo offline
- ✅ Versiones consistentes (sqlx 0.7.4, axum 0.7.5)

### 5. Archivos Nuevos
- ✅ `.env.example` con ejemplo de `DATABASE_URL`

---

## Resultado Final

### ✅ Esquema Cedar Generado Correctamente

```json
{
  "HodeiMVP": {
    "entityTypes": {
      "User": {
        "attributes": {
          "role": { "type": "String", "required": true },
          "tenant_id": { "type": "String" },
          "service": { "type": "String" }
        }
      },
      "Document": {
        "attributes": {
          "owner_id": { "type": "String", "required": true },
          "is_public": { "type": "Boolean", "required": true },
          "tenant_id": { "type": "String" },
          "service": { "type": "String" }
        }
      }
    },
    "actions": {
      "Create": { "appliesTo": { "principalTypes": ["User"], "resourceTypes": ["Document"] } },
      "Read": { "appliesTo": { "principalTypes": ["User"], "resourceTypes": ["Document"] } },
      "Update": { "appliesTo": { "principalTypes": ["User"], "resourceTypes": ["Document"] } },
      "Delete": { "appliesTo": { "principalTypes": ["User"], "resourceTypes": ["Document"] } }
    }
  }
}
```

### ✅ Compilación Exitosa

```bash
$ cargo build
   Compiling hodei_cedar_mvp_kernel v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)
```

### ✅ Schema-as-Code Funcionando

- Agregar una nueva entidad con `#[derive(HodeiEntity)]` → esquema se actualiza automáticamente
- Agregar una nueva acción con `#[derive(HodeiAction)]` → esquema se actualiza automáticamente
- **Cero hardcodeo**: Todo se genera desde el código Rust

---

## Lecciones Aprendidas

### 1. Inventory y el Linker
El crate `inventory` usa constructores estáticos para registrar items. Si el linker no incluye un crate, sus constructores no se ejecutan. Solución: forzar el enlace con `use crate_name as _;`.

### 2. SQLx Compile-Time vs Runtime
- **Compile-time** (`query!`): Seguro pero requiere base de datos en compilación
- **Runtime** (`query`): Flexible pero errores en runtime
- **Offline mode**: Mejor de ambos mundos con metadata pre-generada

### 3. Metaprogramación en Rust
Los macros procedurales con `inventory` son poderosos para schema-as-code, pero requieren entender cómo funciona el enlazado de Rust.

---

## Próximos Pasos Recomendados

1. **Configurar PostgreSQL**:
   ```bash
   cp .env.example .env
   # Editar .env con credenciales reales
   cargo sqlx database create
   cargo sqlx migrate run
   ```

2. **Ejecutar aplicación**:
   ```bash
   cargo run
   ```

3. **Probar endpoints**:
   ```bash
   # Crear documento
   curl -X POST http://localhost:3000/documents \
     -H "Authorization: Bearer alice" \
     -H "Content-Type: application/json" \
     -d '{"resource_id":"test1","is_public":false}'
   ```

4. **Opcional - Habilitar verificación compile-time**:
   ```bash
   # Generar metadata
   cargo sqlx prepare
   
   # Ahora puedes usar query! y query_as! con verificación
   # sin necesidad de DATABASE_URL en cada compilación
   ```

---

## Referencias

- [Inventory Crate Documentation](https://docs.rs/inventory/latest/inventory/)
- [SQLx Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [Rust Forum - Inventory Issue](https://users.rust-lang.org/t/help-with-inventory-crate/25411)
- [SQLx Query Macros](https://docs.rs/sqlx/latest/sqlx/macro.query.html)
