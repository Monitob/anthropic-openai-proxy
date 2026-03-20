# codex-router-rust

Rust implementation of a proxy server that converts between Anthropic API format (used by Claude Code) and OpenAI API format (used by Scaleway Generative APIs), with support for streaming SSE.

## Architecture

```
Claude Code (format Anthropic)
    ↓ HTTP
codex-router-rust (localhost:8787) — conversion Anthropic ↔ OpenAI
    ↓ HTTPS
Scaleway Generative APIs
```

## Prerequisites

- Rust ≥ 1.70
- Cargo (Rust's package manager)

## Quick start

```bash
# Clone the repository
git clone https://github.com/yourusername/codex-router-rust.git
cd codex-router-rust

# Build and run
UPSTREAM_BASE_URL=https://api.scaleway.ai cargo run

# With detailed logging
UPSTREAM_BASE_URL=https://api.scaleway.ai RUST_LOG=info cargo run
```

## Configuration

The server can be configured using environment variables:

| Variable | Required | Default | Description |
|---|---|---|---|
| `UPSTREAM_BASE_URL` | ✅ | — | Base URL for Scaleway (e.g., `https://api.scaleway.ai`) |
| `PORT` | | `8787` | Port to listen on |
| `RUST_LOG` | | `info` | Logging level (trace, debug, info, warn, error) |

## Endpoints

| Path | Method | Description |
|---|---|---|
| `/v1/messages` | POST | Anthropic Messages API → converted and proxied to upstream |
| `/health` | GET | Health check (returns upstream URL) |

## Features

- **Format Conversion**: Converts between Anthropic and OpenAI API formats
- **Streaming Support**: Full SSE streaming support for real-time responses
- **TLS Bypass**: Skips certificate verification for internal CAs
- **Zero Dependencies**: No external npm packages required (Rust equivalent)
- **Robust Error Handling**: Comprehensive error handling and logging

## Implementation Details

The Rust implementation uses:

- **Axum**: Web framework for building the HTTP server
- **Tokio**: Async runtime for handling concurrent requests
- **Hyper**: Low-level HTTP implementation
- **Hyper-TLS**: TLS support with the ability to bypass certificate verification
- **Serde**: Serialization/Deserialization for JSON handling

The code structure follows Rust best practices with proper error handling using `anyhow` and `thiserror` crates.

## Building

```bash
# Build in release mode
cargo build --release

# Build with specific features or target
cargo build --release --target x86_64-unknown-linux-musl

# Build and run in one command
UPSTREAM_BASE_URL=https://api.scaleway.ai cargo run --release
```

## Deployment

### Local Deployment

The application can be deployed locally using cargo:

```bash
# Set required environment variable
export UPSTREAM_BASE_URL=https://api.scaleway.ai

# Run in development mode
cargo run

# Run in production mode with logging
RUST_LOG=info UPSTREAM_BASE_URL=https://api.scaleway.ai cargo run --release
```

### Container Deployment

The application can be containerized using the provided Dockerfile. The container is optimized for production use with a small footprint.

#### Building the Container

```bash
# Build the container image
podman build -t codex-router-rust .

# Or with Docker
docker build -t codex-router-rust .
```

#### Running the Container

```bash
# Run with required environment variables
podman run -d \
    --name codex-router \
    -p 8787:8787 \
    -e UPSTREAM_BASE_URL=https://api.scaleway.ai \
    codex-router-rust

# With custom port
podman run -d \
    --name codex-router \
    -p 3000:8787 \
    -e UPSTREAM_BASE_URL=https://api.scaleway.ai \
    -e PORT=8787 \
    codex-router-rust
```

### Container Features

- **Multi-stage build**: Reduces final image size
- **Non-root user**: Runs as user with UID 1001 for security
- **Alpine Linux**: Small base image for reduced footprint
- **Health check**: Built-in health check for container orchestration
- **Production optimized**: Static binary with minimal dependencies

### Docker Compose (Optional)

```yaml
version: '3.8'

services:
  proxy:
    build: .
    ports:
      - "8787:8787"
    environment:
      - UPSTREAM_BASE_URL=https://api.scaleway.ai
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:8787/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

## Environment Setup

```bash
# Configure Claude Code to use this proxy
export ANTHROPIC_BASE_URL=http://localhost:8787
export ANTHROPIC_MODEL=qwen3.5-397b-a17b
```

The model is passed through directly to the backend - changing `ANTHROPIC_MODEL` is sufficient to switch models.

## Development

```bash
# Run with auto-reload during development
cargo install cargo-watch
cargo watch -x "run"

# Run tests (when implemented)
cargo test
```

## License

MIT License - see LICENSE file for details.

## Docker/Podman Deployment

The application can be containerized using the provided Dockerfile. The container is optimized for production use with a small footprint.

### Building the Container

```bash
# Build the container image
podman build -t codex-router-rust .

# Or with Docker
docker build -t codex-router-rust .
```

### Running the Container

```bash
# Run with required environment variables
podman run -d \
    --name codex-router \
    -p 8787:8787 \
    -e UPSTREAM_BASE_URL=https://api.scaleway.ai \
    codex-router-rust

# With custom port
podman run -d \
    --name codex-router \
    -p 3000:8787 \
    -e UPSTREAM_BASE_URL=https://api.scaleway.ai \
    -e PORT=8787 \
    codex-router-rust
```

### Environment Variables

| Variable | Required | Description |
|---|---|---|
| `UPSTREAM_BASE_URL` | ✅ | Base URL for Scaleway (e.g., `https://api.scaleway.ai`) |
| `PORT` | ❌ | Port to listen on (default: 8787) |

### Container Features

- **Multi-stage build**: Reduces final image size
- **Non-root user**: Runs as user with UID 1001 for security
- **Alpine Linux**: Small base image for reduced footprint
- **Health check**: Built-in health check for container orchestration
- **Production optimized**: Static binary with minimal dependencies

### Docker Compose (Optional)

```yaml
version: '3.8'

services:
  proxy:
    build: .
    ports:
      - "8787:8787"
    environment:
      - UPSTREAM_BASE_URL=https://api.scaleway.ai
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:8787/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```
