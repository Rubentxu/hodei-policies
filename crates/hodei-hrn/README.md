# hodei-authz-sdk-hrn

Core types for the Hodei authorization framework.

## Overview

`hodei-authz-sdk-hrn` provides fundamental types used across the Hodei framework, most notably the **HRN (Hodei Resource Name)** - a resource naming system inspired by AWS ARN.

## Features

- **HRN (Hodei Resource Name)**: Unique identifiers for resources
- **Builder Pattern**: Easy construction of HRNs
- **Serialization**: Full serde support
- **Parsing**: Parse HRNs from strings
- **Zero Dependencies**: No heavy dependencies (only serde and thiserror)

## Installation

```toml
[dependencies]
hodei-authz-sdk-hrn = "0.1"
```

## Usage

### Creating an HRN

```rust
use hodei_hrn::Hrn;

let hrn = Hrn::builder()
    .service("users-api")
    .tenant_id("tenant-123")
    .resource("user/alice")
    .unwrap()
    .build()
    .unwrap();

println!("{}", hrn);
// Output: hrn:hodei-authz-sdk:users-api:global:tenant-123:user/alice
```

### Parsing an HRN

```rust
use hodei_hrn::Hrn;

let hrn: Hrn = "hrn:hodei-authz-sdk:docs:global:tenant-1:document/doc-1"
    .parse()
    .unwrap();

assert_eq!(hrn.service, "docs");
assert_eq!(hrn.tenant_id, "tenant-1");
assert_eq!(hrn.resource_type, "document");
assert_eq!(hrn.resource_id, "doc-1");
```

### Serialization

```rust
use hodei_hrn::Hrn;
use serde_json;

let hrn = Hrn::builder()
    .service("api")
    .tenant_id("t1")
    .resource("user/1")
    .unwrap()
    .build()
    .unwrap();

let json = serde_json::to_string(&hrn).unwrap();
let deserialized: Hrn = serde_json::from_str(&json).unwrap();

assert_eq!(hrn, deserialized);
```

## HRN Format

```
hrn:hodei-authz-sdk:{service}:global:{tenant_id}:{resource_type}/{resource_id}
```

- **partition**: Always "hodei-authz-sdk"
- **service**: Service name (e.g., "users-api", "documents")
- **region**: Always "global" (reserved for future use)
- **tenant_id**: Tenant identifier for multi-tenancy
- **resource_type**: Type of resource (e.g., "user", "document")
- **resource_id**: Unique identifier within the resource type

## Features

### Optional Features

- `sqlx`: Enables PostgreSQL type support for HRN

```toml
[dependencies]
hodei-authz-sdk-hrn = { version = "0.1", features = ["sqlx"] }
```

## License

MIT OR Apache-2.0
