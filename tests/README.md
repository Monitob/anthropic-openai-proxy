# Testing the Anthropic↔OpenAI Format Converter Proxy

This directory contains end-to-end tests for the proxy that verify it works correctly with both OpenAI and Qwen providers.

## Running the Tests

The tests can be run using the provided script, which will start the server with different providers and run the tests.

### Prerequisites

- Rust and Cargo installed
- The proxy application built and ready to run

### Using the Test Script

The easiest way to run the tests is using the provided script:

```bash
# Make the script executable (if not already)
chmod +x tests/run_tests.sh

# Run the tests
./tests/run_tests.sh
```

The script will:
1. Start the server with PROVIDER=openai
2. Run the tests
3. Stop the server
4. Start the server with PROVIDER=qwen
5. Run the tests again
6. Stop the server

### Running Tests Manually

If you prefer to run the tests manually, follow these steps:

1. Start the server with the desired provider:

```bash
# For OpenAI provider
UPSTREAM_BASE_URL=http://localhost:1234 PROVIDER=openai cargo run

# For Qwen provider
UPSTREAM_BASE_URL=http://localhost:1234 PROVIDER=qwen cargo run
```

2. In another terminal, run the tests:

```bash
# Run all tests
cargo test --test basic_test

# Run a specific test
cargo test test_proxy_with_openai_provider
```

## Test Description

The tests in `basic_test.rs` verify the following functionality:

1. **test_proxy_with_openai_provider** - Verifies the proxy can handle requests when configured for the OpenAI provider
2. **test_proxy_with_qwen_provider** - Verifies the proxy can handle requests when configured for the Qwen provider
3. **test_health_check** - Verifies the health check endpoint is working correctly

## Test Environment

The tests assume that an upstream server is not actually running at `http://localhost:1234`. This is intentional, as we're testing that the proxy correctly forwards requests in the proper format, not that it can communicate with a real upstream server. The tests expect either a successful response (if you have a mock server running) or a 502 Bad Gateway response (which is expected when the upstream server is not available).

## Adding New Tests

To add new tests, edit the `basic_test.rs` file. Make sure to use the `#[tokio::test]` attribute for async tests and include appropriate error handling.

## Troubleshooting

If tests are failing:

1. Make sure the server has enough time to start (the tests sleep for 5 seconds)
2. Check that the correct environment variables are set
3. Verify the server is listening on port 8787
4. Check the server logs for any errors

The tests are designed to be robust and should pass regardless of whether an upstream server is actually available, as we're primarily testing the proxy's request forwarding behavior.