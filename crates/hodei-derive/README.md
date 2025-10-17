# hodei-derive

Derive macros for the Hodei authorization framework.

## Overview

This crate provides procedural macros to automatically generate Cedar Policy schemas from Rust types:

- `#[derive(HodeiEntity)]` - Generate entity schemas
- `#[derive(HodeiAction)]` - Generate action schemas

## Usage

```rust
use hodei_derive::{HodeiEntity, HodeiAction};
use hodei_hrn::Hrn;

#[derive(HodeiEntity)]
#[hodei(entity_type = "MyApp::User")]
struct User {
    id: Hrn,
    email: String,
    role: String,
}

#[derive(HodeiAction)]
#[hodei(namespace = "MyApp")]
enum DocumentAction {
    #[hodei(principal = "User", resource = "Document")]
    Read,
    
    #[hodei(principal = "User", resource = "Document")]
    Update,
}
```

## Features

- **Auto-generate Cedar schemas** from Rust types
- **Type-safe** authorization at compile time
- **Inventory integration** for schema discovery
- **Zero runtime overhead**

## Documentation

For more information, see the [Hodei documentation](https://github.com/Rubentxu/hodei-policies).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
