# Build stage
FROM docker.io/library/rust:1.94-bookworm AS builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY crates/auth/Cargo.toml crates/auth/

# Create dummy source files to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN mkdir -p crates/auth/src && echo "pub fn dummy() {}" > crates/auth/src/lib.rs

# Build dependencies only
RUN cargo build --release --workspace

# Now copy the actual source code
COPY . .

# Touch the dummy files to ensure they're newer than the copied source
RUN touch src/main.rs crates/auth/src/lib.rs

# Build the application
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/brain-overflow /app/brain-overflow

# Copy configuration file (optional, can be overridden via volume)
COPY brain-overflow.toml /app/brain-overflow.toml

# Create a non-root user to run the application
RUN useradd -m -u 1000 -s /bin/bash appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 10086

ENTRYPOINT ["/app/brain-overflow"]
