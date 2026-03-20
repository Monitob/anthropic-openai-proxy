FROM rust:1.78-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    gcc \
    openssl-dev \
    pkgconf

# Set working directory
WORKDIR /app

# Copy manifest and lock file
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo 'fn main() { println!("dummy"); }' > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove dummy file
RUN rm -f src/main.rs

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Production stage
FROM alpine:latest

# Install CA certificates
RUN apk --no-cache add ca-certificates

# Create non-root user
RUN addgroup -g 1001 -S appuser && \
    adduser -u 1001 -S appuser -G appuser

# Set working directory
WORKDIR /home/appuser

# Copy the binary from builder stage
COPY --from=builder /app/target/release/codex-router-rust ./codex-router-rust

# Change ownership to non-root user
RUN chown -R 1001:1001 . && \
    chmod +x ./codex-router-rust

# Switch to non-root user
USER 1001

# Expose port
EXPOSE 8787

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD wget -qO- http://localhost:8787/health || exit 1

# Set environment variables
ENV UPSTREAM_BASE_URL=https://api.scaleway.ai

# Run the application
CMD ["./codex-router-rust"]