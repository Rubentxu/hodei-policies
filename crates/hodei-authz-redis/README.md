# hodei-authz-sdk-authz-redis

Redis cache invalidation adapter for the Hodei authorization framework.

## Overview

`hodei-authz-sdk-authz-redis` provides a Redis-based implementation of the `CacheInvalidation` trait from `hodei-authz-sdk-authz`. It enables distributed cache invalidation using Redis Pub/Sub, perfect for multi-instance deployments.

## Features

- **CacheInvalidation Implementation**: Pub/Sub for policy updates
- **Distributed**: Works across multiple application instances
- **Async/Await**: Built on tokio and redis-rs
- **Connection Testing**: Validates Redis connection on startup
- **Error Handling**: Typed errors with detailed messages

## Installation

```toml
[dependencies]
hodei-authz-sdk-authz-redis = "0.1"
redis = { version = "0.32", features = ["tokio-comp"] }
```

## Usage

### Basic Setup

```rust
use hodei_redis::RedisCacheInvalidation;
use hodei_authz::CacheInvalidation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create cache invalidation handler
    let cache = RedisCacheInvalidation::new("redis://localhost:6379").await?;
    
    Ok(())
}
```

### Publishing Invalidation Events

```rust
use hodei_authz::CacheInvalidation;

// After updating a policy, invalidate caches
cache.invalidate_policies().await?;
```

### Subscribing to Invalidation Events

```rust
use hodei_authz::CacheInvalidation;
use std::sync::Arc;

let cache = Arc::new(cache);
let cache_clone = cache.clone();

// Spawn subscriber in background
tokio::spawn(async move {
    cache_clone
        .subscribe_to_invalidations(|| {
            println!("Cache invalidation received! Reloading policies...");
            // Reload policies from database
        })
        .await
});
```

### Complete Example with Policy Store

```rust
use hodei_postgres::PostgresPolicyStore;
use hodei_redis::RedisCacheInvalidation;
use hodei_authz::{PolicyStore, CacheInvalidation};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let pool = PgPool::connect("postgres://...").await?;
    let store = Arc::new(PostgresPolicyStore::new(pool));
    let cache = Arc::new(RedisCacheInvalidation::new("redis://...").await?);
    
    // Load initial policies
    let policy_set = Arc::new(RwLock::new(store.load_all_policies().await?));
    
    // Subscribe to invalidations
    let store_clone = store.clone();
    let policy_set_clone = policy_set.clone();
    
    tokio::spawn(async move {
        cache
            .subscribe_to_invalidations(move || {
                let store = store_clone.clone();
                let policy_set = policy_set_clone.clone();
                
                tokio::spawn(async move {
                    if let Ok(new_policies) = store.load_all_policies().await {
                        *policy_set.write().await = new_policies;
                    }
                });
            })
            .await
    });
    
    // Create a policy (will trigger invalidation)
    let policy_id = store.create_policy("permit(...);".to_string()).await?;
    cache.invalidate_policies().await?;
    
    Ok(())
}
```

## Redis Pub/Sub Channel

The adapter uses the channel: `hodei-authz-sdk:policy:invalidate`

## Error Handling

```rust
use hodei_authz::CacheError;

match cache.invalidate_policies().await {
    Ok(()) => println!("Invalidation published"),
    Err(CacheError::Connection(e)) => eprintln!("Connection error: {}", e),
    Err(CacheError::Publish(e)) => eprintln!("Publish error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

Integration tests require a running Redis instance:

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7

# Run tests
REDIS_URL="redis://localhost:6379" cargo test -- --ignored
```

## License

MIT OR Apache-2.0
