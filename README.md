# Anthropic↔OpenAI Format Converter Proxy

A standalone proxy that converts between Anthropic and OpenAI API formats, with support for multiple upstream providers including OpenAI and Qwen.

## Features

- Converts Anthropic API requests to Scaleway, OpenAI, and Qwen format and vice versa
- Supports both streaming and non-streaming requests
- Configurable upstream provider (Scaleway, OpenAI, Qwen, etc.)
- Comprehensive logging of all incoming requests
- End-to-end testing framework
- Built with Rust for high performance and reliability

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo package manager

### Installation

```bash
cargo build
```

### Configuration

The proxy is configured using environment variables:

- `UPSTREAM_BASE_URL`: The base URL of the upstream provider (required)
- `PROVIDER`: The upstream provider to use (optional, default: "scaleway")
  - "scaleway-qwen" or "qwen" - For Qwen models via Scaleway Generative AI API
  - "scaleway" - For other Scaleway Generative AI models
  - "openai" - For OpenAI-compatible APIs
- `API_KEY`: Your API key for the upstream provider (required)
- `DEFAULT_MODEL`: The default model to use when not specified in requests (optional, defaults to provider-specific models)
  - For Scaleway Qwen: defaults to "qwen72b-chat"
  - For Scaleway: defaults to "mistral-medium"
  - For OpenAI: defaults to "gpt-3.5-turbo"

### Running the Server

```bash
# For Scaleway provider (default)
UPSTREAM_BASE_URL=https://api.scaleway.ai PROVIDER=scaleway API_KEY=your_scaleway_api_key cargo run

# For OpenAI provider
UPSTREAM_BASE_URL=https://api.openai.com PROVIDER=openai API_KEY=your_openai_api_key cargo run

# For Qwen provider
UPSTREAM_BASE_URL=https://huggingface.co/Qwen/Qwen3.5-397B-A17B PROVIDER=qwen API_KEY=your_huggingface_api_key cargo run
```

The server will listen on port 8787 by default.

### Qwen and Scaleway Model Examples

You can use various Qwen models through Scaleway's Generative AI API. Here are examples of how to use them:

```bash
# Qwen models via Scaleway with default model
UPSTREAM_BASE_URL=https://api.scaleway.ai PROVIDER=scaleway-qwen DEFAULT_MODEL=qwen72b-chat API_KEY=your_api_key cargo run

# Use with curl - model will default to qwen72b-chat if not specified
curl http://localhost:8787/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ]
  }'

# You can still override the model in individual requests
curl http://localhost:8787/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen1.8b-chat",
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ]
  }'

# Other Scaleway models with default
UPSTREAM_BASE_URL=https://api.scaleway.ai PROVIDER=scaleway DEFAULT_MODEL=mistral-medium API_KEY=your_api_key cargo run

# Mistral models (will use mistral-medium by default)
curl http://localhost:8787/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ]
  }'

# Or use other Scaleway models
# - "mistral-small"
# - "mistral-tiny"
# - "llama2-70b-chat"
# - "llama2-13b-chat"
```

## API Endpoints

- `POST /v1/messages` - Main endpoint for chat completions
- `GET /health` - Health check endpoint

## End-to-End Testing

The project includes a comprehensive end-to-end testing framework that verifies the proxy works correctly with different providers.

### Running Tests

```bash
# Run all tests with the test script (recommended)
./tests/run_tests.sh

# Run tests manually
UPSTREAM_BASE_URL=http://localhost:1234 PROVIDER=openai cargo test --test basic_test
```

The test script will automatically test both the OpenAI and Qwen providers.

## Logging

The proxy includes comprehensive logging that captures:

- All incoming requests
- Request headers
- Request body content
- Provider being used

Logs are output to stdout with debug level information.

## Architecture

The proxy is built with the following components:

- **Axum**: Web framework for handling HTTP requests
- **Hyper**: HTTP client and server implementation
- **Tokio**: Async runtime
- **Tower/Tower-HTTP**: Middleware framework

## Docker Deployment

The project includes Docker support for easy deployment in containerized environments:

### Simple Deployment

For production use, the recommended approach is to build the binary locally and use the simple Dockerfile:

```bash
# Build the release binary
cargo build --release

# Copy the binary to the target location
cp target/release/codex-router .

# Build the Docker image
docker build -f Dockerfile.simple -t codex-router .
```

### Build with Docker

Alternatively, you can build everything with Docker using the build Dockerfile:

```bash
# Build with Docker
docker build -f Dockerfile.build -t codex-router .
```

### Running with Docker

```bash
# Run with Scaleway provider
docker run -d \
  -p 8787:8787 \
  -e UPSTREAM_BASE_URL=https://api.scaleway.ai \
  -e PROVIDER=scaleway-qwen \
  -e API_KEY=your_api_key \
  -e DEFAULT_MODEL=qwen3.5-397b-a17b \
  --name codex-router \
  codex-router
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.