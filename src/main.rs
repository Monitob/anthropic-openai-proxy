use axum::{
    extract::State,
    http::{self, Request},
    response::{Response, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use std::fmt;

// Custom error type that implements IntoResponse
#[derive(Debug)]
struct AppError(anyhow::Error);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl From<http::Error> for AppError {
    fn from(e: http::Error) -> Self {
        AppError(anyhow::anyhow!("HTTP error: {}", e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError(anyhow::anyhow!("JSON error: {}", e))
    }
}

impl From<hyper::Error> for AppError {
    fn from(e: hyper::Error) -> Self {
        AppError(anyhow::anyhow!("Hyper error: {}", e))
    }
}
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::sync::OnceCell;
use hyper::{Body, client::{Client as HyperClient, HttpConnector}};
use tokio_native_tls;
use hyper_tls::HttpsConnector;
use std::env;

// Static instance of Hyper client with TLS bypass
static HTTP_CLIENT: OnceCell<HyperClient<HttpsConnector<HttpConnector>, Body>> = OnceCell::const_new();

// Configuration state
#[derive(Clone)]
struct AppState {
    upstream_base_url: String,
    https_client: HyperClient<HttpsConnector<HttpConnector>, Body>,
}

// Anthropic API types
#[derive(Deserialize, Debug)]
struct AnthropicMessage {
    role: String,
    content: Value,
}

#[derive(Deserialize, Debug)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Deserialize, Debug)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(default)]
    system: Value,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(default)]
    stream: bool,
}

// OpenAI API types
#[derive(Serialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Serialize, Debug)]
struct OpenAIToolCall {
    id: String,
    r#type: String,
    function: OpenAIFunction,
}

#[derive(Serialize, Debug)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Serialize, Debug)]
struct OpenAITool {
    r#type: String,
    function: OpenAIToolFunction,
}

#[derive(Serialize, Debug)]
struct OpenAIToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    stream: bool,
}

// Anthropic response types
#[derive(Serialize, Debug)]
struct AnthropicResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<ContentBlock>,
    stop_reason: String,
    stop_sequence: Option<String>,
    model: String,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

// SSE event types for streaming
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum SSEEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStart },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize, content_block: ContentBlockStart },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: ContentBlockDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta },
    #[serde(rename = "message_stop")]
    MessageStop,
}

#[derive(Serialize, Debug)]
struct MessageStart {
    id: String,
    r#type: String,
    role: String,
    content: Vec<Value>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Serialize, Debug)]
struct ContentBlockStart {
    #[serde(flatten)]
    content: ContentBlock,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum ContentBlockDelta {
    TextDelta { type_: String, text: String },
    InputJsonDelta { type_: String, partial_json: String },
}

#[derive(Serialize, Debug)]
struct MessageDelta {
    stop_reason: String,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Serialize, Debug)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

// Health check response
type HealthResponse = serde_json::Value;

// Application error

// Initialize the Hyper client with TLS bypass
async fn init_http_client() -> Result<HyperClient<HttpsConnector<HttpConnector>, Body>, AppError> {
    let https = HttpsConnector::new();

    let client = HyperClient::builder()
        .build(https);

    Ok(client)
}

// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(json!({
        "status": "ok",
        "message": "Server is running"
    }))
}

// Convert Anthropic messages to OpenAI format
fn format_anthropic_to_openai(req: &AnthropicRequest) -> OpenAIRequest {
    let mut openai_messages = Vec::new();

    // Process system messages
    let mut system_text = String::new();
    if req.system.is_array() {
        if let Some(system_array) = req.system.as_array() {
            for item in system_array {
                if let Some(text) = item.get("text") {
                    if let Some(text_str) = text.as_str() {
                        if !system_text.is_empty() {
                            system_text.push_str("\n\n");
                        }
                        system_text.push_str(text_str);
                    }
                }
            }
        }
    } else if let Some(text) = req.system.as_str() {
        system_text.push_str(text);
    }

    // Add system message if present
    if !system_text.is_empty() {
        openai_messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: Some(system_text),
            tool_calls: None,
        });
    }

    // Process conversation messages
    for msg in &req.messages {
        match msg.role.as_str() {
            "user" => {
                let mut user_text = String::new();
                let mut tool_messages = Vec::new();

                if let Some(content_array) = msg.content.as_array() {
                    for part in content_array {
                        if let Some(text_part) = part.get("text") {
                            let text = match text_part {
                                Value::String(s) => s.clone(),
                                _ => text_part.to_string(),
                            };
                            if !user_text.is_empty() {
                                user_text.push_str("\n");
                            }
                            user_text.push_str(&text);
                        } else if let Some(tool_result) = part.get("tool_result") {
                            if let Some(_tool_call_id) = tool_result.get("tool_use_id").and_then(|id| id.as_str()) {
                                let content = match tool_result.get("content") {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(other) => other.to_string(),
                                    None => "".to_string(),
                                };

                                tool_messages.push(OpenAIMessage {
                                    role: "tool".to_string(),
                                    content: Some(content),
                                    tool_calls: None,
                                });
                            }
                        }
                    }
                } else if let Some(text) = msg.content.as_str() {
                    user_text.push_str(text);
                }

                // Add user message if present
                if !user_text.is_empty() {
                    openai_messages.push(OpenAIMessage {
                        role: "user".to_string(),
                        content: Some(user_text),
                        tool_calls: None,
                    });
                }

                // Add tool messages
                openai_messages.extend(tool_messages);
            },
            "assistant" => {
                let mut assistant_msg = OpenAIMessage {
                    role: "assistant".to_string(),
                    content: None,
                    tool_calls: None,
                };

                let mut text_content = String::new();
                let mut tool_calls = Vec::new();

                if let Some(content_array) = msg.content.as_array() {
                    for part in content_array {
                        if let Some(text_part) = part.get("text") {
                            let text = match text_part {
                                Value::String(s) => s.clone(),
                                _ => text_part.to_string(),
                            };
                            if !text_content.is_empty() {
                                text_content.push_str("\n");
                            }
                            text_content.push_str(&text);
                        } else if let Some(tool_use) = part.get("tool_use") {
                            if let (Some(id), Some(name), Some(input)) = (
                                tool_use.get("id").and_then(|id| id.as_str()),
                                tool_use.get("name").and_then(|name| name.as_str()),
                                tool_use.get("input")
                            ) {
                                tool_calls.push(OpenAIToolCall {
                                    id: id.to_string(),
                                    r#type: "function".to_string(),
                                    function: OpenAIFunction {
                                        name: name.to_string(),
                                        arguments: input.to_string(),
                                    },
                                });
                            }
                        }
                    }
                }

                // Set text content if present
                if !text_content.is_empty() {
                    assistant_msg.content = Some(text_content);
                }

                // Set tool calls if present
                if !tool_calls.is_empty() {
                    assistant_msg.tool_calls = Some(tool_calls);
                }

                // Only add if there's content or tool calls
                if assistant_msg.content.is_some() || assistant_msg.tool_calls.is_some() {
                    openai_messages.push(assistant_msg);
                }
            },
            _ => {},
        }
    }

    // Convert tools if present
    let openai_tools = req.tools.as_ref().map(|tools| {
        tools.iter().map(|t| OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIToolFunction {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone(),
            },
        }).collect()
    });

    OpenAIRequest {
        model: req.model.clone(),
        messages: openai_messages,
        temperature: req.temperature,
        tools: openai_tools,
        stream: req.stream,
    }
}

// Convert OpenAI completion to Anthropic format (non-streaming)
fn format_openai_to_anthropic(completion: &Value, model: &str) -> AnthropicResponse {
    let mut content = Vec::new();

    // Extract the first choice
    if let Some(choices) = completion.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(message) = choice.get("message") {
                // Handle text content
                if let Some(text_content) = message.get("content").and_then(|c| c.as_str()) {
                    content.push(ContentBlock::Text {
                        text: text_content.to_string(),
                    });
                }

                // Handle tool calls
                if let Some(tool_calls) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
                    for tool_call in tool_calls {
                        if let (Some(id), Some(function)) = (
                            tool_call.get("id").and_then(|i| i.as_str()),
                            tool_call.get("function")
                        ) {
                            if let (Some(name), Some(args)) = (
                                function.get("name").and_then(|n| n.as_str()),
                                function.get("arguments")
                            ) {
                                let input: Value = match args {
                                    Value::String(s) => {
                                        // Try to parse as JSON, fall back to string
                                        serde_json::from_str(s).unwrap_or(Value::String(s.clone()))
                                    },
                                    other => other.clone(),
                                };

                                content.push(ContentBlock::ToolUse {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    input,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Determine stop reason
    let stop_reason = if let Some(choices) = completion.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(finish_reason) = choice.get("finish_reason").and_then(|fr| fr.as_str()) {
                if finish_reason == "tool_calls" {
                    "tool_use".to_string()
                } else {
                    "end_turn".to_string()
                }
            } else {
                "end_turn".to_string()
            }
        } else {
            "end_turn".to_string()
        }
    } else {
        "end_turn".to_string()
    };

    AnthropicResponse {
        id: format!("msg_{}", chrono::Utc::now().timestamp_millis()),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content,
        stop_reason,
        stop_sequence: None,
        model: model.to_string(),
    }
}

// Handle the main messages endpoint
#[axum::debug_handler]
async fn handle_messages(
    State(state): State<AppState>,
    Json(anthropic_req): Json<AnthropicRequest>,
) -> Result<Response<Body>, AppError> {
    // Convert Anthropic request to OpenAI format
    let openai_req = format_anthropic_to_openai(&anthropic_req);

    // Prepare upstream request
    let upstream_url = format!("{}/v1/chat/completions", state.upstream_base_url);
    let openai_body = serde_json::to_vec(&openai_req).map_err(|e| AppError(anyhow::anyhow!("Failed to serialize request: {}", e)))?;

    // Extract bearer token from incoming request
    // In a real implementation, this would come from the request headers
    let bearer_token = "dummy-token"; // This would be extracted from headers in a real implementation

    // Create HTTP request to upstream
    let http_request = Request::builder()
        .uri(&upstream_url)
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", bearer_token))
        .body(Body::from(openai_body))?;

    // Send request to upstream
    // The request is sent but we're not actually using the response yet
    // In a real implementation, we would process the response
    state.https_client.request(http_request).await.map_err(|e| AppError(anyhow::anyhow!("Request failed: {}", e)))?;

    // Check if we're streaming
    if anthropic_req.stream {
        // For streaming, we need to convert the OpenAI SSE stream to Anthropic SSE format
        // This is a simplified version - in a real implementation, we'd need to parse and transform the SSE stream

        // Create a streaming response
        let (mut sender, body) = hyper::Body::channel();

        // Spawn a task to handle the streaming conversion
        tokio::spawn(async move {

            // Send message_start event
            let message_start = SSEEvent::MessageStart {
                message: MessageStart {
                    id: format!("msg_{}", chrono::Utc::now().timestamp_millis()),
                    r#type: "message".to_string(),
                    role: "assistant".to_string(),
                    content: Vec::new(),
                    model: openai_req.model.clone(),
                    stop_reason: None,
                    stop_sequence: None,
                    usage: Usage {
                        input_tokens: 1,
                        output_tokens: 1,
                    },
                },
            };

            if let Ok(json) = serde_json::to_string(&message_start) {
                let _ = sender.send_data(format!("event: message_start\ndata: {}\n\n", json).into()).await;
            }

            // In a real implementation, we would read from the upstream SSE stream and transform events
            // For now, we'll send a simple text delta as an example
            let content_block_start = SSEEvent::ContentBlockStart {
                index: 0,
                content_block: ContentBlockStart {
                    content: ContentBlock::Text { text: "".to_string() },
                },
            };

            if let Ok(json) = serde_json::to_string(&content_block_start) {
                let _ = sender.send_data(format!("event: content_block_start\ndata: {}\n\n", json).into()).await;
            }

            // Example text delta
            let content_block_delta = SSEEvent::ContentBlockDelta {
                index: 0,
                delta: ContentBlockDelta::TextDelta {
                    type_: "text_delta".to_string(),
                    text: "Hello from Rust proxy!".to_string(),
                },
            };

            if let Ok(json) = serde_json::to_string(&content_block_delta) {
                let _ = sender.send_data(format!("event: content_block_delta\ndata: {}\n\n", json).into()).await;
            }

            // Close the content block
            let content_block_stop = SSEEvent::ContentBlockStop { index: 0 };

            if let Ok(json) = serde_json::to_string(&content_block_stop) {
                let _ = sender.send_data(format!("event: content_block_stop\ndata: {}\n\n", json).into()).await;
            }

            // Send message_delta with stop reason
            let message_delta = SSEEvent::MessageDelta {
                delta: MessageDelta {
                    stop_reason: "end_turn".to_string(),
                    stop_sequence: None,
                    usage: Usage {
                        input_tokens: 100,
                        output_tokens: 150,
                    },
                },
            };

            if let Ok(json) = serde_json::to_string(&message_delta) {
                let _ = sender.send_data(format!("event: message_delta\ndata: {}\n\n", json).into()).await;
            }

            // Send message_stop
            let message_stop = SSEEvent::MessageStop;

            if let Ok(json) = serde_json::to_string(&message_stop) {
                let _ = sender.send_data(format!("event: message_stop\ndata: {}\n\n", json).into()).await;
            }

            // Close the sender
            let _ = sender.send_trailers(hyper::HeaderMap::new()).await;
        });

        Ok(Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "text/event-stream")
            .header(http::header::CACHE_CONTROL, "no-cache")
            .header(http::header::CONNECTION, "keep-alive")
            .body(body)?)
    } else {
        // For non-streaming, we can wait for the full response and convert it
        // This is a simplified version - we're not actually making the upstream call yet

        // In a real implementation, we would wait for the full response from the upstream
        // For now, we'll return a placeholder response
        let anthropic_response = AnthropicResponse {
            id: format!("msg_{}", chrono::Utc::now().timestamp_millis()),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ContentBlock::Text {
                text: "Response from Rust proxy".to_string(),
            }],
            stop_reason: "end_turn".to_string(),
            stop_sequence: None,
            model: openai_req.model,
        };

        Ok(Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&anthropic_response)?))?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    // Get upstream base URL from environment variable
    let upstream_base_url = env::var("UPSTREAM_BASE_URL")
        .map_err(|_| anyhow::anyhow!("UPSTREAM_BASE_URL environment variable is required"))?;

    // Initialize HTTP client
    let https_client = init_http_client().await.map_err(|e| anyhow::anyhow!("Failed to initialize HTTP client: {}", e.0))?;

    // Create shared application state
    let app_state = AppState {
        upstream_base_url,
        https_client,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/v1/messages", post(handle_messages))
        .route("/health", get(health_check))
        .with_state(app_state.clone());

    // Run our application
    let port = env::var("PORT").unwrap_or_else(|_| "8787".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;

    println!("🚀 qwen3.5-scw-router listening on http://{}", addr);
    println!("   Upstream: {}", app_state.upstream_base_url);

    let server = hyper::Server::bind(&addr).serve(app.into_make_service());
    server.await?;

    Ok(())
}
