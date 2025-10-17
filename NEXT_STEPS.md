# 🚀 Próximos Pasos - Hodei Framework

**Estado Actual**: ✅ Fase 1 Completada - Framework compilando correctamente

---

## ✅ Lo que Acabamos de Lograr

1. **Estructura de Framework Profesional**
   - 7 crates independientes
   - Workspace configurado
   - Compilación exitosa

2. **Documentación Completa**
   - 6 documentos técnicos
   - Estrategia de testing
   - Plan de implementación

3. **Base Sólida**
   - Código funcional
   - Macros funcionando
   - Listo para publicar (kernel, derive, core)

---

## 🎯 Siguiente Sesión: Fase 2

### Objetivo: Implementar Adapters (6-8h)

#### 1. hodei-authz-sdk-authz-postgres (2-3h)

**Extraer de**: `crates/app/src/auth.rs` líneas 50-150

**Implementar**:
```rust
// crates/hodei-authz-sdk-authz-postgres/src/lib.rs
use async_trait::async_trait;
use hodei_authz::PolicyStore;
use sqlx::PgPool;

pub struct PostgresPolicyStore {
    pool: PgPool,
}

#[async_trait]
impl PolicyStore for PostgresPolicyStore {
    async fn create_policy(&self, content: String) -> Result<String> {
        // Copiar implementación de app/src/auth.rs
    }
    
    async fn get_policy(&self, id: &str) -> Result<Option<String>> {
        // Copiar implementación
    }
    
    // ... otros métodos
}
```

**Incluir migraciones**:
```sql
-- crates/hodei-authz-sdk-authz-postgres/migrations/001_create_policies.sql
CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
);
```

#### 2. hodei-authz-sdk-authz-redis (2-3h)

**Extraer de**: `crates/app/src/auth.rs` líneas 200-250

**Implementar**:
```rust
// crates/hodei-authz-sdk-authz-redis/src/lib.rs
use async_trait::async_trait;
use hodei_authz::CacheInvalidation;

pub struct RedisCacheInvalidation {
    client: redis::Client,
}

#[async_trait]
impl CacheInvalidation for RedisCacheInvalidation {
    async fn invalidate_policies(&self) -> Result<()> {
        // Copiar implementación de app/src/auth.rs
    }
}
```

#### 3. hodei-authz-sdk-authz-axum (2-3h)

**Crear desde cero**:
```rust
// crates/hodei-authz-sdk-authz-axum/src/extractors.rs
use axum::{async_trait, extract::FromRequestParts};
use hodei_authz::HodeiEntity;

pub struct AuthenticatedUser<T: HodeiEntity>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for AuthenticatedUser<T>
where
    T: HodeiEntity + DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Implementar extracción de usuario
    }
}
```

---

## 📝 Comandos para Siguiente Sesión

```bash
# 1. Verificar estado actual
cd /home/rubentxu/Proyectos/rust/hodei-authz-sdk-policies
cargo check --workspace

# 2. Extraer PostgresPolicyStore
# Copiar código de crates/app/src/auth.rs a crates/hodei-authz-sdk-authz-postgres/src/lib.rs

# 3. Extraer RedisCacheInvalidation
# Copiar código de crates/app/src/auth.rs a crates/hodei-authz-sdk-authz-redis/src/lib.rs

# 4. Definir traits en hodei-authz-sdk-core
# Crear crates/hodei-authz-sdk-core/src/traits.rs

# 5. Compilar y verificar
cargo check --workspace

# 6. Ejecutar tests
cargo test --workspace
```

---

## 🧪 Tests a Implementar

### hodei-authz-sdk-kernel
```rust
#[test]
fn test_hrn_builder() {
    let hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    
    assert_eq!(hrn.service, "test");
}
```

### hodei-authz-sdk-authz-postgres
```rust
#[tokio::test]
async fn test_policy_store() {
    let pool = create_test_pool().await;
    let store = PostgresPolicyStore::new(pool);
    
    let id = store.create_policy("permit(...);".to_string())
        .await
        .unwrap();
    
    let policy = store.get_policy(&id).await.unwrap();
    assert!(policy.is_some());
}
```

---

## 📦 Orden de Publicación en crates.io

Cuando esté listo:

```bash
# 1. hodei-authz-sdk-kernel (sin dependencias del workspace)
cd crates/hodei-authz-sdk-kernel
cargo publish --dry-run
cargo publish

# 2. hodei-authz-sdk-derive (depende de kernel)
cd ../hodei-authz-sdk-derive
cargo publish

# 3. hodei-authz-sdk-core (depende de kernel + derive)
cd ../hodei-authz-sdk-core
cargo publish

# 4. hodei-authz-sdk-authz-postgres (depende de core)
cd ../hodei-authz-sdk-authz-postgres
cargo publish

# 5. hodei-authz-sdk-authz-redis (depende de core)
cd ../hodei-authz-sdk-authz-redis
cargo publish

# 6. hodei-authz-sdk-authz-axum (depende de core)
cd ../hodei-authz-sdk-authz-axum
cargo publish

# 7. hodei-authz-sdk (meta-crate, depende de todos)
cd ../hodei-authz-sdk
cargo publish
```

---

## 🎯 Objetivos de Próxima Sesión

1. ✅ Implementar hodei-authz-sdk-authz-postgres completo
2. ✅ Implementar hodei-authz-sdk-authz-redis completo
3. ✅ Implementar hodei-authz-sdk-authz-axum básico
4. ✅ Agregar tests unitarios
5. ✅ Verificar que todo compila
6. ⏰ (Opcional) Crear primer ejemplo

**Tiempo estimado**: 6-8 horas

---

## 📚 Referencias Útiles

- **FRAMEWORK_DESIGN.md** - Arquitectura completa
- **FRAMEWORK_EVOLUTION.md** - Cómo evolucionamos el código
- **TESTING_STRATEGY.md** - Estrategia de tests
- **REFACTORING_COMPLETE.md** - Estado actual

---

## ✅ Checklist Rápido

Antes de empezar próxima sesión:

- [ ] Leer FRAMEWORK_EVOLUTION.md sección "Fase 2"
- [ ] Revisar crates/app/src/auth.rs para extraer código
- [ ] Tener PostgreSQL y Redis corriendo (docker-compose)
- [ ] Verificar que compila: `cargo check --workspace`

---

**¡Excelente progreso! El framework está tomando forma.** 🚀
