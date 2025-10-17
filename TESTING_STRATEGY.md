# 🧪 Estrategia de Testing - Hodei Framework

**Objetivo**: Cobertura completa de tests para garantizar calidad del framework

---

## 📊 Niveles de Testing

### 1. Unit Tests (Por Crate)
### 2. Integration Tests (Entre Crates)
### 3. End-to-End Tests (Framework Completo)
### 4. Property-Based Tests (Invariantes)
### 5. Benchmark Tests (Performance)

---

## 🎯 Tests por Crate

### hodei-authz-sdk-kernel

**Tests Unitarios**:

```rust
// crates/hodei-authz-sdk-kernel/src/hrn.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hrn_builder_basic() {
        let hrn = Hrn::builder()
            .service("test-service")
            .tenant_id("tenant-1")
            .resource("user/123")
            .unwrap()
            .build()
            .unwrap();
        
        assert_eq!(hrn.service, "test-service");
        assert_eq!(hrn.tenant_id, "tenant-1");
        assert_eq!(hrn.resource_id, "123");
    }

    #[test]
    fn test_hrn_to_string() {
        let hrn = Hrn::builder()
            .service("documents-api")
            .tenant_id("tenant-a")
            .resource("document/doc-1")
            .unwrap()
            .build()
            .unwrap();
        
        let expected = "hrn:hodei-authz-sdk:documents-api:global:tenant-a:document/doc-1";
        assert_eq!(hrn.to_string(), expected);
    }

    #[test]
    fn test_hrn_from_string() {
        let hrn_str = "hrn:hodei-authz-sdk:users-api:global:tenant-b:user/alice";
        let hrn = Hrn::from_str(hrn_str).unwrap();
        
        assert_eq!(hrn.service, "users-api");
        assert_eq!(hrn.tenant_id, "tenant-b");
        assert_eq!(hrn.resource_type, "user");
        assert_eq!(hrn.resource_id, "alice");
    }

    #[test]
    fn test_hrn_validation_invalid_service() {
        let result = Hrn::builder()
            .service("invalid service")  // Espacios no permitidos
            .tenant_id("tenant-1")
            .resource("user/123")
            .unwrap()
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_hrn_serialization() {
        let hrn = Hrn::builder()
            .service("test")
            .tenant_id("t1")
            .resource("r/1")
            .unwrap()
            .build()
            .unwrap();
        
        let json = serde_json::to_string(&hrn).unwrap();
        let deserialized: Hrn = serde_json::from_str(&json).unwrap();
        
        assert_eq!(hrn, deserialized);
    }

    #[test]
    fn test_request_context_creation() {
        let ctx = RequestContext::new("tenant-1");
        
        assert_eq!(ctx.tenant_id, "tenant-1");
        assert!(ctx.timestamp.is_some());
    }

    #[test]
    fn test_request_context_with_custom_fields() {
        let mut ctx = RequestContext::new("tenant-1");
        ctx.custom.insert(
            "user_role".to_string(),
            serde_json::json!("admin")
        );
        
        assert_eq!(
            ctx.custom.get("user_role"),
            Some(&serde_json::json!("admin"))
        );
    }
}
```

**Property-Based Tests**:

```rust
// crates/hodei-authz-sdk-kernel/tests/property_tests.rs
use proptest::prelude::*;
use hodei_hrn::Hrn;

proptest! {
    #[test]
    fn test_hrn_roundtrip(
        service in "[a-z-]{3,20}",
        tenant in "[a-z0-9-]{3,20}",
        resource_type in "[a-z]{3,10}",
        resource_id in "[a-z0-9-]{1,20}"
    ) {
        let hrn = Hrn::builder()
            .service(&service)
            .tenant_id(&tenant)
            .resource(&format!("{}/{}", resource_type, resource_id))
            .unwrap()
            .build()
            .unwrap();
        
        let hrn_str = hrn.to_string();
        let parsed = Hrn::from_str(&hrn_str).unwrap();
        
        prop_assert_eq!(hrn, parsed);
    }
}
```

---

### hodei-authz-sdk-derive

**Tests de Macros**:

```rust
// crates/hodei-authz-sdk-derive/tests/entity_derive_tests.rs
use hodei_derive::HodeiEntity;
use hodei_hrn::Hrn;
use serde::{Serialize, Deserialize};

#[derive(HodeiEntity, Serialize, Deserialize, Clone)]
#[hodei-authz-sdk(entity_type = "Test::User")]
struct TestUser {
    id: Hrn,
    name: String,
    age: i32,
}

#[test]
fn test_entity_derive_basic() {
    let hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    
    let user = TestUser {
        id: hrn.clone(),
        name: "Alice".to_string(),
        age: 30,
    };
    
    assert_eq!(user.hodei_type_name(), "Test::User");
    assert_eq!(user.hodei_hrn(), &hrn);
}

#[test]
fn test_entity_to_cedar() {
    let hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    
    let user = TestUser {
        id: hrn,
        name: "Bob".to_string(),
        age: 25,
    };
    
    let cedar_entity = user.to_cedar_entity();
    
    // Verificar que tiene los atributos correctos
    assert!(cedar_entity.attrs().contains_key("name"));
    assert!(cedar_entity.attrs().contains_key("age"));
    assert!(cedar_entity.attrs().contains_key("tenant_id"));
}

#[derive(HodeiEntity, Serialize, Deserialize, Clone)]
#[hodei-authz-sdk(entity_type = "Test::Document")]
struct TestDocument {
    id: Hrn,
    
    #[entity_type = "Test::User"]
    owner_id: Hrn,
    
    is_public: bool,
}

#[test]
fn test_entity_with_hrn_field() {
    let doc_hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("doc/1")
        .unwrap()
        .build()
        .unwrap();
    
    let owner_hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    
    let doc = TestDocument {
        id: doc_hrn,
        owner_id: owner_hrn,
        is_public: false,
    };
    
    let cedar_entity = doc.to_cedar_entity();
    
    // Verificar que owner_id es un EntityUid, no un String
    let owner_attr = cedar_entity.attrs().get("owner_id").unwrap();
    // Debería ser un Entity type
}
```

**Tests de Action Derive**:

```rust
// crates/hodei-authz-sdk-derive/tests/action_derive_tests.rs
use hodei_derive::HodeiAction;
use hodei_hrn::{Hrn, RequestContext};

#[derive(HodeiAction)]
#[hodei-authz-sdk(namespace = "Test")]
enum TestCommand {
    #[hodei-authz-sdk(principal = "User", resource = "Document", creates_resource)]
    Create(CreatePayload),
    
    #[hodei-authz-sdk(principal = "User", resource = "Document")]
    Read { id: Hrn },
    
    #[hodei-authz-sdk(principal = "User", resource = "Document")]
    Delete { id: Hrn },
}

#[derive(Clone)]
struct CreatePayload {
    resource_id: String,
}

impl CreatePayload {
    fn to_virtual_entity(&self, _ctx: &RequestContext) -> cedar_policy::Entity {
        // Implementación de prueba
        todo!()
    }
}

#[test]
fn test_action_derive_basic() {
    let hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("doc/1")
        .unwrap()
        .build()
        .unwrap();
    
    let action = TestCommand::Read { id: hrn };
    
    // Verificar que genera el action name correcto
    let euid = action.to_cedar_action_euid();
    assert_eq!(euid.to_string(), "Action::\"Document::Read\"");
}

#[test]
fn test_action_creates_resource() {
    let create = TestCommand::Create(CreatePayload {
        resource_id: "new-doc".to_string(),
    });
    
    assert!(create.creates_resource_from_payload());
    
    let hrn = Hrn::builder()
        .service("test")
        .tenant_id("t1")
        .resource("doc/1")
        .unwrap()
        .build()
        .unwrap();
    
    let read = TestCommand::Read { id: hrn };
    assert!(!read.creates_resource_from_payload());
}
```

---

### hodei-authz-sdk-core

**Tests Unitarios del Mapper**:

```rust
// crates/hodei-authz-sdk-core/tests/mapper_tests.rs
use hodei_authz::{HodeiMapperService, HodeiEntity, HodeiAction};
use hodei_hrn::{Hrn, RequestContext};

#[test]
fn test_mapper_with_existing_resource() {
    let principal = create_test_user();
    let action = TestCommand::Read { id: test_hrn() };
    let resource = create_test_document();
    let context = RequestContext::new("tenant-1");
    
    let result = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        Some(&resource),
        &context,
        None,
    );
    
    assert!(result.is_ok());
    let (request, entities) = result.unwrap();
    
    // Verificar que el request tiene los componentes correctos
    assert_eq!(request.principal(), &principal.to_cedar_euid());
    assert_eq!(request.resource(), &resource.to_cedar_euid());
}

#[test]
fn test_mapper_with_virtual_resource() {
    let principal = create_test_user();
    let action = TestCommand::Create(CreatePayload {
        resource_id: "new-doc".to_string(),
    });
    let context = RequestContext::new("tenant-1");
    
    let result = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        None,  // No resource from DB
        &context,
        None,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_mapper_missing_resource_error() {
    let principal = create_test_user();
    let action = TestCommand::Read { id: test_hrn() };
    let context = RequestContext::new("tenant-1");
    
    let result = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        None,  // Missing required resource
        &context,
        None,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_mapper_with_cedar_context() {
    let principal = create_test_user();
    let action = TestCommand::Read { id: test_hrn() };
    let resource = create_test_document();
    let context = RequestContext::new("tenant-1");
    
    let cedar_context = serde_json::json!({
        "ip_address": "192.168.1.1",
        "user_agent": "test-agent"
    });
    
    let result = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        Some(&resource),
        &context,
        Some(cedar_context),
    );
    
    assert!(result.is_ok());
}
```

**Tests del AuthorizationService**:

```rust
// crates/hodei-authz-sdk-core/tests/authorization_service_tests.rs
use hodei_authz::{AuthorizationService, PolicyStore};
use std::sync::Arc;

#[tokio::test]
async fn test_authorization_service_creation() {
    let policy_store = MockPolicyStore::new();
    let cache = MockCache::new();
    let schema = create_test_schema();
    
    let service = AuthorizationService::new(
        policy_store,
        cache,
        &schema,
    ).await;
    
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_authorization_allow() {
    let service = create_test_service().await;
    
    let principal = create_test_user();
    let action = TestCommand::Read { id: test_hrn() };
    let resource = create_test_document_owned_by(&principal);
    let context = RequestContext::new("tenant-1");
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        Some(&resource),
        &context,
        None,
    ).unwrap();
    
    let decision = service.is_authorized(request, &entities).await;
    
    assert!(decision.is_allow());
}

#[tokio::test]
async fn test_authorization_deny() {
    let service = create_test_service().await;
    
    let principal = create_test_user();
    let action = TestCommand::Delete { id: test_hrn() };
    let resource = create_test_document_owned_by_other();
    let context = RequestContext::new("tenant-1");
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &principal,
        &action,
        Some(&resource),
        &context,
        None,
    ).unwrap();
    
    let decision = service.is_authorized(request, &entities).await;
    
    assert!(decision.is_deny());
}

#[tokio::test]
async fn test_policy_crud() {
    let service = create_test_service().await;
    
    // Create
    let policy_content = "permit(principal, action, resource);";
    let policy_id = service.create_policy(policy_content.to_string())
        .await
        .unwrap();
    
    // Read
    let retrieved = service.get_policy(&policy_id).await.unwrap();
    assert_eq!(retrieved, Some(policy_content.to_string()));
    
    // Update
    let new_content = "forbid(principal, action, resource);";
    service.update_policy(&policy_id, new_content.to_string())
        .await
        .unwrap();
    
    let updated = service.get_policy(&policy_id).await.unwrap();
    assert_eq!(updated, Some(new_content.to_string()));
    
    // Delete
    service.delete_policy(&policy_id).await.unwrap();
    let deleted = service.get_policy(&policy_id).await.unwrap();
    assert_eq!(deleted, None);
}
```

---

### hodei-authz-sdk-authz-postgres

**Tests de Integración con TestContainers**:

```rust
// crates/hodei-authz-sdk-authz-postgres/tests/integration_tests.rs
use hodei_postgres::PostgresPolicyStore;
use hodei_authz::PolicyStore;
use testcontainers::{clients, images};

#[tokio::test]
async fn test_postgres_policy_store() {
    // Iniciar contenedor PostgreSQL
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());
    
    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );
    
    let pool = sqlx::PgPool::connect(&connection_string)
        .await
        .unwrap();
    
    let store = PostgresPolicyStore::new(pool);
    
    // Ejecutar migraciones
    store.migrate().await.unwrap();
    
    // Test CRUD
    let policy_id = store.create_policy(
        "permit(principal, action, resource);".to_string()
    ).await.unwrap();
    
    let retrieved = store.get_policy(&policy_id).await.unwrap();
    assert!(retrieved.is_some());
    
    let all_policies = store.list_policies().await.unwrap();
    assert_eq!(all_policies.len(), 1);
}

#[tokio::test]
async fn test_postgres_concurrent_writes() {
    let store = create_test_store().await;
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let store = store.clone();
            tokio::spawn(async move {
                store.create_policy(
                    format!("permit(principal, action, resource) /* {} */;", i)
                ).await
            })
        })
        .collect();
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let policies = store.list_policies().await.unwrap();
    assert_eq!(policies.len(), 10);
}
```

---

### hodei-authz-sdk-authz-redis

**Tests de Cache Invalidation**:

```rust
// crates/hodei-authz-sdk-authz-redis/tests/cache_tests.rs
use hodei_redis::RedisCacheInvalidation;
use hodei_authz::CacheInvalidation;
use testcontainers::{clients, images};

#[tokio::test]
async fn test_redis_cache_invalidation() {
    let docker = clients::Cli::default();
    let redis = docker.run(images::redis::Redis::default());
    
    let redis_url = format!(
        "redis://localhost:{}",
        redis.get_host_port_ipv4(6379)
    );
    
    let cache = RedisCacheInvalidation::new(&redis_url)
        .await
        .unwrap();
    
    // Test invalidation
    cache.invalidate_policies().await.unwrap();
}

#[tokio::test]
async fn test_redis_pubsub() {
    let cache = create_test_cache().await;
    
    let received = Arc::new(Mutex::new(false));
    let received_clone = received.clone();
    
    cache.subscribe_to_invalidations(move || {
        *received_clone.lock().unwrap() = true;
    }).await.unwrap();
    
    cache.invalidate_policies().await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    assert!(*received.lock().unwrap());
}
```

---

## 🔄 Tests de Integración End-to-End

```rust
// tests/e2e_tests.rs
use hodei-authz-sdk::prelude::*;
use hodei_postgres::PostgresPolicyStore;
use hodei_redis::RedisCacheInvalidation;

#[tokio::test]
async fn test_complete_authorization_flow() {
    // Setup
    let (pg_pool, redis_url) = setup_test_infrastructure().await;
    
    let policy_store = PostgresPolicyStore::new(pg_pool);
    policy_store.migrate().await.unwrap();
    
    let cache = RedisCacheInvalidation::new(&redis_url).await.unwrap();
    
    let schema = generate_test_schema();
    let auth_service = AuthorizationService::new(
        policy_store,
        cache,
        &schema,
    ).await.unwrap();
    
    // Create policy
    let policy = r#"
        permit(
            principal,
            action == Action::"Document::Read",
            resource
        ) when {
            resource.owner_id == principal
        };
    "#;
    
    auth_service.create_policy(policy.to_string()).await.unwrap();
    
    // Test authorization
    let user = create_test_user();
    let document = create_test_document_owned_by(&user);
    let action = DocumentCommand::Read { id: document.id.clone() };
    let context = RequestContext::new("tenant-1");
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user,
        &action,
        Some(&document),
        &context,
        None,
    ).unwrap();
    
    let decision = auth_service.is_authorized(request, &entities).await;
    
    assert!(decision.is_allow());
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let service = create_test_service().await;
    
    // User from tenant-a
    let user_a = create_user_in_tenant("tenant-a");
    // Document from tenant-b
    let doc_b = create_document_in_tenant("tenant-b");
    
    let action = DocumentCommand::Read { id: doc_b.id.clone() };
    let context = RequestContext::new("tenant-a");
    
    let (request, entities) = HodeiMapperService::build_auth_package(
        &user_a,
        &action,
        Some(&doc_b),
        &context,
        None,
    ).unwrap();
    
    let decision = service.is_authorized(request, &entities).await;
    
    // Debe denegar por diferente tenant
    assert!(decision.is_deny());
}
```

---

## 📊 Cobertura de Tests

### Objetivo de Cobertura

```
hodei-authz-sdk-kernel:     > 90%
hodei-authz-sdk-derive:     > 85% (macros son difíciles de testear)
hodei-authz-sdk-core:       > 90%
hodei-authz-sdk-authz-postgres:   > 85%
hodei-authz-sdk-authz-redis:      > 85%
hodei-authz-sdk-authz-axum:       > 80%
hodei-authz-sdk (meta):     > 70% (principalmente re-exports)
```

### Comandos de Testing

```bash
# Todos los tests
cargo test --all-features --workspace

# Tests con cobertura
cargo tarpaulin --all-features --workspace --out Html

# Tests específicos de un crate
cargo test -p hodei-authz-sdk-core --all-features

# Tests de integración
cargo test --test '*' --all-features

# Property-based tests
cargo test --features proptest --all-features

# Benchmarks
cargo bench --all-features
```

---

## ⏱️ Benchmarks

```rust
// benches/authorization_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hodei-authz-sdk::prelude::*;

fn bench_authorization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = rt.block_on(create_test_service());
    
    c.bench_function("authorize_simple", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (request, entities) = create_test_request();
                service.is_authorized(request, &entities).await
            })
        })
    });
}

criterion_group!(benches, bench_authorization);
criterion_main!(benches);
```

---

## ✅ Checklist de Testing

- [ ] Unit tests para cada módulo
- [ ] Integration tests entre crates
- [ ] E2E tests del framework completo
- [ ] Property-based tests para invariantes
- [ ] Benchmarks de performance
- [ ] Tests con TestContainers (PostgreSQL, Redis)
- [ ] Tests de concurrencia
- [ ] Tests de multi-tenancy
- [ ] Tests de schema generation
- [ ] Tests de derive macros
- [ ] Cobertura > 85% en todos los crates core
- [ ] CI/CD ejecutando todos los tests

---

**Próximo**: Implementar estos tests mientras refactorizamos el código
