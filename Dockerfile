# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY kernel/Cargo.toml ./kernel/
COPY hodei_provider/Cargo.toml ./hodei_provider/
COPY hodei_provider_derive/Cargo.toml ./hodei_provider_derive/
COPY hodei_domain/Cargo.toml ./hodei_domain/

# Create dummy source files to cache dependencies
RUN mkdir -p src kernel/src hodei_provider/src hodei_provider_derive/src hodei_domain/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > kernel/src/lib.rs && \
    echo "pub fn dummy() {}" > hodei_provider/src/lib.rs && \
    echo "pub fn dummy() {}" > hodei_provider_derive/src/lib.rs && \
    echo "pub fn dummy() {}" > hodei_domain/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src kernel/src hodei_provider/src hodei_provider_derive/src hodei_domain/src

# Copy source code
COPY . .

# Build application with schema-discovery
RUN cargo build --release --features schema-discovery

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and schema
COPY --from=builder /app/target/release/hodei_cedar_mvp_kernel /app/hodei_app
COPY --from=builder /app/cedar_schema.json /app/cedar_schema.json

# Expose port
EXPOSE 3000

# Run application
CMD ["/app/hodei_app"]
