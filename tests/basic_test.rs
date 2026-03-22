use std::net::TcpListener;
use tokio::time::{sleep, Duration};
use reqwest;
use serde_json::json;

// This test requires a running server on port 8787
// Start the server with: UPSTREAM_BASE_URL=http://localhost:1234 cargo run
// The test will verify that the proxy correctly forwards requests

#[tokio::test]
async fn test_proxy_with_openai_provider() {
    // This test assumes the proxy is running with OpenAI provider
    // Set environment variables:
    // UPSTREAM_BASE_URL=http://localhost:1234
    // PROVIDER=openai
    
    // Give the server time to start
    sleep(Duration::from_secs(1)).await;
    
    // Create a client
    let client = reqwest::Client::new();
    
    // Send a request to the proxy
    let response = client
        .post("http://localhost:8787/v1/messages")
        .json(&json!({
            "model": "claude-2",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, how are you?"
                }
            ]
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    // The request should be proxied, even if the upstream is not available
    // We're just testing that the proxy forwards correctly
    assert!(response.status().is_success() || response.status() == reqwest::StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn test_proxy_with_qwen_provider() {
    // This test assumes the proxy is running with Qwen provider
    // Set environment variables:
    // UPSTREAM_BASE_URL=http://localhost:1234
    // PROVIDER=qwen
    
    // Give the server time to start
    sleep(Duration::from_secs(1)).await;
    
    // Create a client
    let client = reqwest::Client::new();
    
    // Send a request to the proxy
    let response = client
        .post("http://localhost:8787/v1/messages")
        .json(&json!({
            "model": "Qwen-1.8B",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, how are you?"
                }
            ]
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    // The request should be proxied, even if the upstream is not available
    // We're just testing that the proxy forwards correctly
    assert!(response.status().is_success() || response.status() == reqwest::StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn test_health_check() {
    // Give the server time to start
    sleep(Duration::from_secs(1)).await;
    
    // Create a client
    let client = reqwest::Client::new();
    
    // Send a request to the health check endpoint
    let response = client
        .get("http://localhost:8787/health")
        .send()
        .await
        .expect("Failed to send request");
    
    // The health check should always succeed
    assert!(response.status().is_success());
    
    // Verify the response contains the expected data
    let body = response.json::<serde_json::Value>().await.expect("Failed to parse response");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "Server is running");
}