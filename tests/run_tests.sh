#!/bin/bash

# Script to run end-to-end tests for the proxy

# Function to start the server
start_server() {
    local provider=${1:-scaleway}
    echo "Starting server with PROVIDER=$provider"
    
    # Start the server in the background
    UPSTREAM_BASE_URL=http://localhost:1234 \
    PROVIDER=$provider \
    API_KEY=test_key \
    cargo run --quiet &
    
    # Store the process ID
    SERVER_PID=$!
    
    # Wait for the server to start
    echo "Waiting for server to start..."
    sleep 5
    
    # Check if server is running
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "Server failed to start"
        exit 1
    fi
    
    echo "Server started with PID $SERVER_PID"
}

# Function to stop the server
stop_server() {
    if [ ! -z "${SERVER_PID+x}" ]; then
        echo "Stopping server with PID $SERVER_PID"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Set up trap to stop server on exit
trap stop_server EXIT

# Test with Scaleway Qwen provider
echo "=== Testing with Scaleway Qwen provider ==="
start_server "scaleway-qwen"

# Run the tests
if cargo test --test basic_test; then
    echo "Scaleway Qwen provider tests passed"
else
    echo "Scaleway Qwen provider tests failed"
    exit 1
fi

# Test with Scaleway provider
echo "=== Testing with Scaleway provider ==="
start_server "scaleway"

# Run the tests
if cargo test --test basic_test; then
    echo "Scaleway provider tests passed"
else
    echo "Scaleway provider tests failed"
    exit 1
fi

# Stop the server
stop_server

# Wait a bit before starting next server
sleep 2

# Test with OpenAI provider
echo "=== Testing with OpenAI provider ==="
start_server "openai"

# Run the tests
if cargo test --test basic_test; then
    echo "OpenAI provider tests passed"
else
    echo "OpenAI provider tests failed"
    exit 1
fi

# Stop the server
stop_server

# Wait a bit before starting next server
sleep 2

# Test with Qwen provider
echo "=== Testing with Qwen provider ==="
start_server "qwen"

# Run the tests
if cargo test --test basic_test; then
    echo "Qwen provider tests passed"
    exit 0
else
    echo "Qwen provider tests failed"
    exit 1
fi

echo "All tests completed successfully!"
