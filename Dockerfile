# Multi-stage Dockerfile for testing cocos4-rust
# This ensures feature parity with Cocos4

# =============================================================================
# Stage 1: Builder - Compile and test the Rust code
# =============================================================================
FROM rust:1.75-bookworm AS builder

WORKDIR /usr/src/cocos4-rust

# Install additional dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build the project (release mode for optimizations)
RUN cargo build --release --verbose

# Run tests
RUN cargo test --release --verbose

# =============================================================================
# Stage 2: Runtime - Minimal image for running tests
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /usr/src/cocos4-rust

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts from builder
COPY --from=builder /usr/src/cocos4-rust/target/release/libcocos4_rust.rlib /usr/lib/
COPY --from=builder /usr/src/cocos4-rust/target/release/deps/* /tmp/deps/

# Copy test runner script
COPY scripts/test-runner.sh /usr/local/bin/test-runner
RUN chmod +x /usr/local/bin/test-runner

# Default command
CMD ["test-runner"]

# =============================================================================
# Stage 3: Development - Full development environment
# =============================================================================
FROM rust:1.75-bookworm AS development

WORKDIR /usr/src/cocos4-rust

# Install development tools
RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    git \
    curl \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools
RUN cargo install cargo-watch cargo-audit cargo-outdated

# Copy source code
COPY . .

# Default command for development
CMD ["cargo", "watch", "-x", "test"]
