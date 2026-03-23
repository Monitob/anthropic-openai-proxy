FROM docker.io/rust:1.82-alpine as builder

# Install required packages
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml .

# Create a dummy src directory for build
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs

# Download dependencies
RUN cargo build --release

# Remove the dummy src directory
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Production stage
FROM alpine:latest

# Install CA certificates
RUN apk --no-cache add ca-certificates

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/codex-router .

# Expose port 8787
EXPOSE 8787

# Required environment variables
ENV API_KEY=""

# Set the entrypoint
ENTRYPOINT ["./codex-router"]