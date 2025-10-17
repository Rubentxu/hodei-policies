# hodei-authz-sdk-authz-postgres

PostgreSQL adapter for the Hodei authorization framework.

## Overview

`hodei-authz-sdk-authz-postgres` provides a production-ready PostgreSQL implementation of the `PolicyStore` trait from `hodei-authz-sdk-authz`. It handles policy persistence, CRUD operations, and includes database migrations.

## Features

- **PolicyStore Implementation**: Full CRUD for Cedar policies
- **Database Migrations**: Automatic schema management with sqlx
- **UUID Generation**: Unique policy identifiers
- **Error Handling**: Typed errors with detailed messages
- **Async/Await**: Built on tokio and sqlx

## Installation

```toml
[dependencies]
hodei-authz-sdk-authz-postgres = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }
```

## Usage

### Basic Setup

```rust
use hodei_postgres::PostgresPolicyStore;
use hodei_authz::PolicyStore;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection pool
    let pool = PgPool::connect("postgres://user:pass@localhost/db").await?;
    
    // Create policy store
    let store = PostgresPolicyStore::new(pool);
    
    // Run migrations
    store.migrate().await?;
    
    Ok(())
}
```

### Creating Policies

```rust
use hodei_authz::PolicyStore;

let policy_content = r#"
permit(
    principal == User::"alice",
    action == Action::"read",
    resource
);
"#;

let policy_id = store.create_policy(policy_content.to_string()).await?;
println!("Created policy: {}", policy_id);
```

### Loading Policies

```rust
use hodei_authz::PolicyStore;

// Load all policies as a Cedar PolicySet
let policy_set = store.load_all_policies().await?;

// Use with Cedar Authorizer
use cedar_policy::{Authorizer, Request, Entities};

let authorizer = Authorizer::new();
let decision = authorizer.is_authorized(&request, &policy_set, &entities);
```

### CRUD Operations

```rust
use hodei_authz::PolicyStore;

// Get a policy
let policy = store.get_policy(&policy_id).await?;

// List all policies
let policies = store.list_policies().await?;
for (id, content) in policies {
    println!("{}: {}", id, content);
}

// Update a policy
store.update_policy(&policy_id, new_content).await?;

// Delete a policy
store.delete_policy(&policy_id).await?;
```

## Database Schema

The migration creates the following table:

```sql
CREATE TABLE policies (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
);
```

## Migrations

Migrations are embedded in the binary and run automatically:

```rust
let store = PostgresPolicyStore::new(pool);
store.migrate().await?;
```

## Error Handling

```rust
use hodei_authz::PolicyStoreError;

match store.get_policy("invalid-id").await {
    Ok(Some(policy)) => println!("Found: {}", policy),
    Ok(None) => println!("Not found"),
    Err(PolicyStoreError::Database(e)) => eprintln!("DB error: {}", e),
    Err(PolicyStoreError::NotFound(id)) => eprintln!("Policy {} not found", id),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

Integration tests require a running PostgreSQL instance:

```bash
# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# Run tests
DATABASE_URL="postgres://postgres:postgres@localhost/test" cargo test -- --ignored
```

## License

MIT OR Apache-2.0
