# Hodei Framework Examples

This directory contains examples demonstrating how to use the Hodei authorization framework.

## Running Examples

```bash
# Basic usage (HRN and core concepts)
cargo run --example basic_usage

# Complete application example
cd crates/app-example
cargo run
```

## Examples

### 1. basic_usage.rs

Demonstrates fundamental concepts:
- Creating HRNs (Hodei Resource Names)
- Defining entities and resources
- Cedar policy syntax
- HRN operations (parsing, serialization)

**Run**:
```bash
cargo run --example basic_usage
```

### 2. app-example/ (Complete Application)

A full-featured example application showing:
- PostgreSQL integration for policy storage
- Redis integration for cache invalidation
- Axum web framework integration
- Multi-tenancy with HRNs
- Authorization service implementation
- Domain entities (Users, Documents, Artifacts)

**Run**:
```bash
cd crates/app-example
docker-compose up -d  # Start PostgreSQL and Redis
cargo run
```

## What Each Example Teaches

| Example | Concepts | Dependencies |
|---------|----------|--------------|
| `basic_usage` | HRN, entities, policies | hodei-authz-sdk-hrn |
| `app-example` | Full stack, web app | All hodei-authz-sdk crates |

## Prerequisites

### For basic_usage
- Rust 1.70+
- No external services needed

### For app-example
- Rust 1.70+
- Docker (for PostgreSQL and Redis)
- Or: PostgreSQL 15+ and Redis 7+ installed locally

## Environment Variables

For `app-example`:

```bash
# Database
DATABASE_URL="postgres://postgres:postgres@localhost:5432/hodei-authz-sdk"

# Redis
REDIS_URL="redis://localhost:6379"

# Server
HOST="0.0.0.0"
PORT="3000"
```

## Next Steps

After running the examples:

1. **Read the documentation**: Check each crate's README
2. **Explore the code**: Look at `app-example/src/` for patterns
3. **Write policies**: Experiment with Cedar policy syntax
4. **Build your app**: Use Hodei in your own project

## Resources

- [Cedar Policy Language](https://docs.cedarpolicy.com/)
- [Hodei Documentation](https://docs.rs/hodei-authz-sdk)
- [API Reference](https://docs.rs/hodei-authz-sdk)

## License

MIT OR Apache-2.0
