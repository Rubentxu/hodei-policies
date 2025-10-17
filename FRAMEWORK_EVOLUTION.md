# 🚀 Hodei Framework - Evolución desde el Código Actual

**Basado en**: Implementación actual funcional  
**Objetivo**: Convertir en framework reutilizable manteniendo lo que funciona

---

## 📊 Estado Actual - Lo que YA FUNCIONA

### ✅ Crates Existentes (Funcionales)

```
crates/
├── kernel/                    ✅ LISTO - Solo necesita renombrar
│   ├── Hrn (builder pattern)
│   ├── Serialización
│   └── Validación
│
├── hodei_provider_derive/     ✅ LISTO - Solo renombrar a hodei-authz-sdk-derive
│   ├── #[derive(HodeiEntity)]
│   ├── #[derive(HodeiAction)]
│   ├── Schema generation
│   └── Inventory system
│
├── hodei_provider/            ✅ LISTO - Renombrar a hodei-authz-sdk-core
│   ├── RuntimeHodeiEntityMapper
│   ├── RuntimeHodeiActionMapper
│   ├── EntitySchemaFragment
│   └── ActionSchemaFragment
│
├── hodei_domain/              ⚠️ EJEMPLO - Mover a examples/
│   ├── Document, Artifact, User
│   ├── Commands
│   └── DTOs
│
└── app/                       ⚠️ EXTRAER COMPONENTES
    ├── AuthorizationService   ✅ Mover a hodei-authz-sdk-core
    ├── HodeiMapperService     ✅ Mover a hodei-authz-sdk-core
    ├── auth.rs                ✅ Extraer traits
    └── main.rs                ⚠️ Dejar como ejemplo
```

### ✅ Lo que Funciona Perfectamente

1. **Derive Macros** - Sistema de metaprogramación completo
2. **Schema Generation** - Generación automática de esquemas Cedar
3. **HRN System** - Sistema de identificación de recursos
4. **Authorization Flow** - Flujo completo de autorización
5. **Multi-tenancy** - Aislamiento por tenant funcionando
6. **Policy Management** - CRUD de políticas dinámicas

---

## 🎯 Plan de Evolución (Basado en lo Actual)

### Fase 1: Reorganización (Sin Cambios de Código)

**Duración**: 2-3 horas  
**Objetivo**: Renombrar y reorganizar manteniendo funcionalidad

#### 1.1 Renombrar Crates

```bash
# Renombrar directorios
mv crates/kernel crates/hodei-authz-sdk-kernel
mv crates/hodei_provider_derive crates/hodei-authz-sdk-derive  
mv crates/hodei_provider crates/hodei-authz-sdk-core
mv crates/hodei_domain examples/domain-example

# Actualizar Cargo.toml
```

**Cambios en Cargo.toml**:

```toml
# Antes
[package]
name = "hodei_provider_derive"

# Después
[package]
name = "hodei-authz-sdk-derive"
version = "0.1.0"
edition = "2021"
description = "Derive macros for Hodei authorization framework"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/hodei-authz-sdk"
keywords = ["authorization", "cedar", "iam", "macro"]
```

#### 1.2 Actualizar Imports

```rust
// Antes
use hodei_provider::{RuntimeHodeiEntityMapper, RuntimeHodeiActionMapper};
use hodei_provider_derive::{HodeiEntity, HodeiAction};

// Después
use hodei_authz::{HodeiEntity as EntityTrait, HodeiAction as ActionTrait};
use hodei_derive::{HodeiEntity, HodeiAction};
```

### Fase 2: Extraer Componentes Reutilizables

**Duración**: 4-5 horas  
**Objetivo**: Mover código de app/ a hodei-authz-sdk-core

#### 2.1 Mover AuthorizationService

**Archivo actual**: `crates/app/src/auth.rs`  
**Destino**: `crates/hodei-authz-sdk-core/src/service.rs`

**Cambios**:

```rust
// crates/hodei-authz-sdk-core/src/service.rs
use async_trait::async_trait;
use cedar_policy::{Authorizer, Decision, Entities, Request, Schema};
use std::sync::Arc;

/// Trait for policy storage backends
#[async_trait]
pub trait PolicyStore: Send + Sync {
    async fn create_policy(&self, content: String) -> Result<String, PolicyStoreError>;
    async fn get_policy(&self, id: &str) -> Result<Option<String>, PolicyStoreError>;
    async fn list_policies(&self) -> Result<Vec<(String, String)>, PolicyStoreError>;
    async fn update_policy(&self, id: &str, content: String) -> Result<(), PolicyStoreError>;
    async fn delete_policy(&self, id: &str) -> Result<(), PolicyStoreError>;
    async fn load_all_policies(&self) -> Result<Vec<String>, PolicyStoreError>;
}

/// Trait for cache invalidation
#[async_trait]
pub trait CacheInvalidation: Send + Sync {
    async fn invalidate_policies(&self) -> Result<(), CacheError>;
}

/// Authorization service (generic over PolicyStore)
pub struct AuthorizationService<P: PolicyStore, C: CacheInvalidation> {
    policy_store: Arc<P>,
    cache_invalidation: Arc<C>,
    authorizer: Authorizer,
    schema: Arc<Schema>,
}

impl<P: PolicyStore, C: CacheInvalidation> AuthorizationService<P, C> {
    pub async fn new(
        policy_store: P,
        cache_invalidation: C,
        schema_json: &str,
    ) -> Result<Self, AuthServiceError> {
        let schema = Schema::from_json_str(schema_json)?;
        let policies = policy_store.load_all_policies().await?;
        
        let mut policy_set = cedar_policy::PolicySet::new();
        for (idx, content) in policies.iter().enumerate() {
            let policy = cedar_policy::Policy::parse(
                Some(format!("policy-{}", idx)),
                content,
            )?;
            policy_set.add(policy)?;
        }
        
        let authorizer = Authorizer::new();
        
        Ok(Self {
            policy_store: Arc::new(policy_store),
            cache_invalidation: Arc::new(cache_invalidation),
            authorizer,
            schema: Arc::new(schema),
        })
    }
    
    pub async fn is_authorized(
        &self,
        request: Request,
        entities: &Entities,
    ) -> Decision {
        self.authorizer
            .is_authorized(&request, &self.policy_set, entities)
            .decision()
    }
    
    // ... métodos de gestión de políticas
}
```

#### 2.2 Mover HodeiMapperService

**Archivo actual**: `crates/app/src/mapper.rs`  
**Destino**: `crates/hodei-authz-sdk-core/src/mapper.rs`

```rust
// crates/hodei-authz-sdk-core/src/mapper.rs
use cedar_policy::{Context, Entities, Entity, Request};
use hodei_hrn::{Hrn, RequestContext};

pub struct HodeiMapperService;

impl HodeiMapperService {
    /// Build authorization package from principal, action, resource
    pub fn build_auth_package<P, A, R>(
        principal: &P,
        action: &A,
        resource: Option<&R>,
        context: &RequestContext,
        cedar_context: Option<serde_json::Value>,
    ) -> Result<(Request, Entities), MapperError>
    where
        P: crate::HodeiEntity,
        A: crate::HodeiAction,
        R: crate::HodeiEntity,
    {
        let mut entities_vec = Vec::new();
        
        // Add principal
        entities_vec.push(principal.to_cedar_entity());
        
        // Add resource or virtual resource
        if let Some(res) = resource {
            entities_vec.push(res.to_cedar_entity());
        } else if action.creates_resource() {
            if let Some(virtual_entity) = action.get_virtual_entity(context) {
                entities_vec.push(virtual_entity);
            }
        }
        
        let entities = Entities::from_entities(
            entities_vec,
            None::<&[Entity]>,
        )?;
        
        // Build Cedar context
        let ctx = if let Some(json) = cedar_context {
            Context::from_json_value(json, None)?
        } else {
            Context::empty()
        };
        
        // Build request
        let request = Request::new(
            principal.to_cedar_euid(),
            action.to_cedar_action_euid(),
            resource
                .map(|r| r.to_cedar_euid())
                .or_else(|| {
                    action.get_virtual_entity(context)
                        .map(|e| e.uid().clone())
                })
                .ok_or(MapperError::MissingResource)?,
            ctx,
            None,
        )?;
        
        Ok((request, entities))
    }
}
```

#### 2.3 Actualizar Traits en hodei-authz-sdk-core

```rust
// crates/hodei-authz-sdk-core/src/traits.rs
use cedar_policy::{Entity, EntityUid};
use hodei_hrn::{Hrn, RequestContext};

/// Trait for entities (principals and resources)
pub trait HodeiEntity: Send + Sync {
    fn entity_type(&self) -> &'static str;
    fn entity_id(&self) -> &Hrn;
    fn to_cedar_entity(&self) -> Entity;
    
    fn to_cedar_euid(&self) -> EntityUid {
        EntityUid::from_type_name_and_id(
            self.entity_type().parse().unwrap(),
            self.entity_id().to_string().parse().unwrap(),
        )
    }
}

/// Trait for actions
pub trait HodeiAction: Send + Sync {
    fn action_name(&self) -> &'static str;
    fn to_cedar_action_euid(&self) -> EntityUid;
    fn creates_resource(&self) -> bool;
    fn get_virtual_entity(&self, ctx: &RequestContext) -> Option<Entity>;
}
```

### Fase 3: Crear Adapters como Crates Separados

**Duración**: 3-4 horas

#### 3.1 hodei-authz-sdk-authz-postgres

**Extraer de**: `crates/app/src/auth.rs` (PostgresAdapter)

```rust
// crates/hodei-authz-sdk-authz-postgres/src/lib.rs
use async_trait::async_trait;
use hodei_authz::{PolicyStore, PolicyStoreError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresPolicyStore {
    pool: PgPool,
}

impl PostgresPolicyStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Run migrations (incluidas en el crate)
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
    }
}

#[async_trait]
impl PolicyStore for PostgresPolicyStore {
    async fn create_policy(&self, content: String) -> Result<String, PolicyStoreError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO policies (id, content, created_at) VALUES ($1, $2, NOW())")
            .bind(&id)
            .bind(&content)
            .execute(&self.pool)
            .await
            .map_err(|e| PolicyStoreError::Database(e.to_string()))?;
        Ok(id)
    }
    
    async fn load_all_policies(&self) -> Result<Vec<String>, PolicyStoreError> {
        let policies = sqlx::query_scalar::<_, String>(
            "SELECT content FROM policies ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PolicyStoreError::Database(e.to_string()))?;
        Ok(policies)
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

CREATE INDEX idx_policies_created_at ON policies(created_at);
```

#### 3.2 hodei-authz-sdk-authz-redis

**Extraer de**: `crates/app/src/auth.rs` (RedisCacheInvalidation)

```rust
// crates/hodei-authz-sdk-authz-redis/src/lib.rs
use async_trait::async_trait;
use hodei_authz::{CacheInvalidation, CacheError};
use redis::aio::MultiplexedConnection;

pub struct RedisCacheInvalidation {
    client: redis::Client,
}

impl RedisCacheInvalidation {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING").query_async::<_, ()>(&mut conn).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CacheInvalidation for RedisCacheInvalidation {
    async fn invalidate_policies(&self) -> Result<(), CacheError> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
            
        redis::cmd("PUBLISH")
            .arg("hodei-authz-sdk:policy:invalidate")
            .arg("reload")
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| CacheError::Publish(e.to_string()))?;
            
        Ok(())
    }
}
```

### Fase 4: Crear hodei-authz-sdk-authz-axum (Integración Web)

**Duración**: 5-6 horas  
**Objetivo**: Facilitar uso en aplicaciones Axum

```rust
// crates/hodei-authz-sdk-authz-axum/src/lib.rs
pub mod extractors;
pub mod middleware;
pub mod error;

pub use extractors::AuthenticatedUser;
pub use middleware::AuthorizationLayer;
pub use error::AuthError;
```

```rust
// crates/hodei-authz-sdk-authz-axum/src/extractors.rs
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use hodei_authz::HodeiEntity;
use serde::de::DeserializeOwned;

/// Extractor for authenticated users
pub struct AuthenticatedUser<T: HodeiEntity>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for AuthenticatedUser<T>
where
    T: HodeiEntity + DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract Bearer token
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingToken)?;
        
        // In real implementation, validate token and fetch user
        // For now, this is a placeholder
        let token = bearer.token();
        
        // TODO: Implement actual user fetching logic
        // This would typically involve:
        // 1. Validate JWT token
        // 2. Extract user ID
        // 3. Fetch user from database
        // 4. Return AuthenticatedUser(user)
        
        Err(AuthError::InvalidToken)
    }
}
```

```rust
// crates/hodei-authz-sdk-authz-axum/src/middleware.rs
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use hodei_authz::AuthorizationService;
use std::sync::Arc;

/// Middleware layer for authorization
pub struct AuthorizationLayer<P, C> {
    service: Arc<AuthorizationService<P, C>>,
}

impl<P, C> AuthorizationLayer<P, C> {
    pub fn new(service: Arc<AuthorizationService<P, C>>) -> Self {
        Self { service }
    }
}

// Middleware function
pub async fn authorize_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    // Extract authorization info from request
    // Validate with AuthorizationService
    // If authorized, call next
    // If not, return 403
    
    next.run(req).await
}
```

### Fase 5: Crear Meta-Crate (hodei-authz-sdk)

**Duración**: 1-2 horas

```rust
// crates/hodei-authz-sdk/src/lib.rs
//! # Hodei Authorization Framework
//!
//! Cedar Policy-based authorization for Rust applications
//!
//! ## Quick Start
//!
//! ```rust
//! use hodei-authz-sdk::prelude::*;
//!
//! #[derive(HodeiEntity)]
//! #[hodei-authz-sdk(entity_type = "MyApp::User")]
//! struct User {
//!     id: Hrn,
//!     email: String,
//! }
//! ```

#![doc = include_str!("../README.md")]

// Re-export all public APIs
pub use hodei_hrn as kernel;
pub use hodei_authz as core;
pub use hodei_derive::{HodeiEntity, HodeiAction};

// Convenience re-exports
pub use hodei_hrn::{Hrn, RequestContext};
pub use hodei_authz::{
    HodeiEntity as EntityTrait,
    HodeiAction as ActionTrait,
    AuthorizationService,
    PolicyStore,
    CacheInvalidation,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use hodei_derive::{HodeiEntity, HodeiAction};
    pub use hodei_hrn::{Hrn, RequestContext};
    pub use hodei_authz::{
        HodeiEntity,
        HodeiAction,
        AuthorizationService,
        PolicyStore,
        CacheInvalidation,
    };
}
```

---

## 📦 Estructura Final del Workspace

```
hodei-authz-sdk/
├── Cargo.toml                        # Workspace
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
│
├── crates/
│   ├── hodei-authz-sdk-kernel/                 # ✅ Renombrar kernel/
│   ├── hodei-authz-sdk-core/                   # ✅ Renombrar hodei_provider/ + extraer de app/
│   ├── hodei-authz-sdk-derive/                 # ✅ Renombrar hodei_provider_derive/
│   ├── hodei-authz-sdk-authz-postgres/               # ✅ Nuevo (extraer de app/)
│   ├── hodei-authz-sdk-authz-redis/                  # ✅ Nuevo (extraer de app/)
│   ├── hodei-authz-sdk-authz-axum/                   # ✅ Nuevo
│   └── hodei-authz-sdk/                        # ✅ Nuevo (meta-crate)
│
├── examples/
│   ├── basic/                        # Ejemplo simple
│   ├── multi-tenant/                 # Multi-tenancy (actual hodei_domain)
│   └── full-app/                     # App completa (actual app/)
│
└── docs/
    ├── guide.md
    └── migration.md
```

---

## 🎯 Cambios Mínimos Necesarios

### En el Código Actual

1. **Renombrar crates** (sin cambiar código)
2. **Mover archivos** entre crates
3. **Actualizar imports**
4. **Hacer traits genéricos** (PolicyStore, CacheInvalidation)
5. **Documentar** con rustdoc

### Lo que NO Cambia

- ✅ Derive macros siguen igual
- ✅ HRN sigue igual
- ✅ Lógica de autorización sigue igual
- ✅ Schema generation sigue igual
- ✅ Tests siguen funcionando

---

## ⏱️ Timeline Realista

| Fase | Duración | Puede Hacerse |
|------|----------|---------------|
| 1. Reorganización | 2-3h | ✅ Ahora |
| 2. Extraer componentes | 4-5h | ✅ Ahora |
| 3. Crear adapters | 3-4h | ✅ Ahora |
| 4. hodei-authz-sdk-authz-axum | 5-6h | ⏰ Después |
| 5. Meta-crate | 1-2h | ✅ Ahora |
| 6. Ejemplos | 4-6h | ⏰ Después |
| 7. Docs | 6-8h | ⏰ Después |
| **MVP** | **15-20h** | **1-2 días** |
| **Completo** | **35-45h** | **1 semana** |

---

## 🚀 Próximo Paso Inmediato

¿Quieres que empiece con la **Fase 1** (reorganización y renombrado)? Puedo:

1. Crear el nuevo workspace structure
2. Renombrar los crates existentes
3. Actualizar todos los imports
4. Verificar que todo compila

Esto tomaría 2-3 horas y tendríamos la base lista para publicar.
