# Hodei Authorization Framework

Cedar Policy-based authorization framework for Rust applications, inspired by AWS IAM.

## Overview

Hodei is a complete authorization framework that provides:

- **Cedar Policy Engine**: Policy-based access control using Amazon's Cedar
- **Multi-Tenancy**: Built-in tenant isolation with HRN (Hodei Resource Name)
- **Derive Macros**: Automatic code generation for entities and actions
- **Database Adapters**: PostgreSQL and Redis support out of the box
- **Web Integration**: Axum middleware and extractors
- **Type-Safe**: Leverages Rust's type system for compile-time safety

## Quick Start

### Installation

```toml
[dependencies]
hodei-authz-sdk = { version = "0.1", features = ["full"] }
```

### Define Your Domain

```rust
use hodei-authz-sdk::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(HodeiEntity, Serialize, Deserialize, Clone)]
#[hodei-authz-sdk(entity_type = "MyApp::User")]
struct User {
    id: Hrn,
    email: String,
    role: String,
}

#[derive(HodeiEntity, Serialize, Deserialize, Clone)]
#[hodei-authz-sdk(entity_type = "MyApp::Document")]
struct Document {
    id: Hrn,
    #[entity_type = "MyApp::User"]
    owner_id: Hrn,
    title: String,
}

#[derive(HodeiAction)]
#[hodei-authz-sdk(namespace = "MyApp")]
enum DocumentCommand {
    #[hodei-authz-sdk(principal = "User", resource = "Document")]
    Read { id: Hrn },
    
    #[hodei-authz-sdk(principal = "User", resource = "Document")]
    Update { id: Hrn },
    
    #[hodei-authz-sdk(principal = "User", resource = "Document", creates_resource)]
    Create { title: String },
}
```

### Write Cedar Policies

```cedar
// Only document owner can read
permit(
    principal,
    action == Action::"Document::Read",
    resource
) when {
    resource.owner_id == principal
};

// Admins can do anything
permit(
    principal,
    action,
    resource
) when {
    principal.role == "admin"
};
```

### Use in Your Application

```rust
use hodei_postgres::PostgresPolicyStore;
use hodei_redis::RedisCacheInvalidation;
use hodei_authz::{PolicyStore, CacheInvalidation};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup database
    let pool = PgPool::connect("postgres://...").await?;
    let store = PostgresPolicyStore::new(pool);
    store.migrate().await?;
    
    // Setup cache invalidation
    let cache = RedisCacheInvalidation::new("redis://...").await?;
    
    // Load policies
    let policy_set = store.load_all_policies().await?;
    
    // Check authorization
    use cedar_policy::{Authorizer, Request, Entities};
    
    let authorizer = Authorizer::new();
    let decision = authorizer.is_authorized(&request, &policy_set, &entities);
    
    if decision.decision() == Decision::Allow {
        println!("Access granted!");
    }
    
    Ok(())
}
```

## Features

### Core Features

- `default` - Core functionality (kernel + core + derive)
- `postgres` - PostgreSQL adapter
- `redis` - Redis cache invalidation
- `axum` - Axum web framework integration
- `full` - All features enabled

```toml
# Minimal installation
[dependencies]
hodei-authz-sdk = "0.1"

# With database support
[dependencies]
hodei-authz-sdk = { version = "0.1", features = ["postgres", "redis"] }

# Everything
[dependencies]
hodei-authz-sdk = { version = "0.1", features = ["full"] }
```

## Architecture

Hodei is composed of several crates:

- **hodei-authz-sdk-hrn**: Core types (HRN)
- **hodei-authz-sdk-derive**: Derive macros
- **hodei-authz-sdk-authz**: Traits and logic
- **hodei-authz-sdk-authz-postgres**: PostgreSQL adapter
- **hodei-authz-sdk-authz-redis**: Redis adapter
- **hodei-authz-sdk-authz-axum**: Axum integration
- **hodei-authz-sdk**: Meta-crate (this crate)

## Examples

### Basic Authorization

```rust
use hodei-authz-sdk::prelude::*;

let user = User {
    id: Hrn::builder()
        .service("myapp")
        .tenant_id("tenant-1")
        .resource("user/alice")
        .unwrap()
        .build()
        .unwrap(),
    email: "alice@example.com".to_string(),
    role: "user".to_string(),
};

// Check if user can read document
let can_read = check_authorization(
    &user,
    DocumentCommand::Read { id: document.id.clone() },
    &document,
).await?;
```

### With Axum

```rust
use hodei_axum::AuthenticatedUser;
use axum::{Router, routing::get, Json};

async fn get_document(
    AuthenticatedUser(user): AuthenticatedUser<User>,
    Path(id): Path<String>,
) -> Result<Json<Document>, StatusCode> {
    // Authorization is handled by middleware
    let document = fetch_document(&id).await?;
    Ok(Json(document))
}

let app = Router::new()
    .route("/documents/:id", get(get_document))
    .layer(middleware::from_fn(authorize_middleware));
```

## Documentation

- [Getting Started Guide](https://docs.rs/hodei-authz-sdk)
- [API Documentation](https://docs.rs/hodei-authz-sdk)
- [Examples](https://github.com/yourusername/hodei-authz-sdk/tree/main/examples)

## License

MIT OR Apache-2.0
