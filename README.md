# Anthropic↔OpenAI Format Converter Proxy

A standalone proxy that converts between Anthropic and OpenAI API formats, with support for multiple upstream providers including OpenAI and Qwen.

## Features

- Converts Anthropic API requests to OpenAI format and vice versa
- Supports both streaming and non-streaming requests
- Configurable upstream provider (OpenAI, Qwen, etc.)
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
- `PROVIDER`: The upstream provider to use (optional, default: "openai")
  - "openai" - For OpenAI-compatible APIs
  - "qwen" - For Qwen model API

### Running the Server

```bash
# For OpenAI provider (default)
UPSTREAM_BASE_URL=https://api.openai.com cargo run

# For Qwen provider
UPSTREAM_BASE_URL=https://huggingface.co/Qwen/Qwen3.5-397B-A17B PROVIDER=qwen cargo run
```

The server will listen on port 8787 by default.

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

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.