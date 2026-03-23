#!/bin/bash

# Source environment variables from .bashrc if it exists
if [ -f "$HOME/.bashrc" ]; then
    source "$HOME/.bashrc"
fi

# Default configuration for Scaleway provider
export UPSTREAM_BASE_URL="https://api.scaleway.ai"
export PROVIDER="scaleway-qwen"
export DEFAULT_MODEL="qwen3.5-397b-a17b"

# Check if API_KEY is set
if [ -z "$API_KEY" ]; then
    echo "Error: API_KEY environment variable is not set"
    echo "Please set your Scaleway API key and try again:"
    echo "export API_KEY=your_actual_key_here"
    exit 1
fi

# Display configuration
echo "Starting proxy with Scaleway provider..."
echo "Upstream URL: $UPSTREAM_BASE_URL"
echo "Provider: $PROVIDER"
echo "Default model: $DEFAULT_MODEL"

# Build the project
RUST_LOG=info cargo build --release

# Run the proxy server
RUST_LOG=info cargo run --release

# Check if cargo run was successful
if [ $? -eq 0 ]; then
    echo "Proxy is running!"
    echo "Health check: curl http://localhost:8787/health"
else
    echo "Failed to start proxy"
    exit 1
fi
