use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::Instrument;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "A misconfigured HTTP client is fatal; fail fast at startup like the other AI statics."
    )]
    reqwest::Client::builder()
        .build()
        .expect("Failed to build reqwest client for review agent")
});

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system")]
    System { content: String },
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    #[serde(rename = "tool")]
    Tool {
        tool_call_id: String,
        content: String,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Serialize, Clone, Debug)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: AssistantMsg,
}

#[derive(Deserialize, Debug)]
struct AssistantMsg {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

// ── Shared types ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

// ── Public result ───────────────────────────────────────────────────────────

pub struct ChatResult {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

// ── API call ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    tools: Vec<Tool>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// Send a chat-completions request to DeepSeek. Returns the model's content
/// and/or tool calls.
#[tracing::instrument(
    skip(messages, tools),
    fields(category = "llm")
)]
pub async fn chat(
    messages: &[Message],
    tools: &[Tool],
) -> Result<ChatResult, Box<dyn std::error::Error + Send + Sync>> {
    let request = ChatRequest {
        model: crate::data::ai::DEFAULT_MODEL.to_string(),
        messages: messages.to_vec(),
        tools: tools.to_vec(),
        temperature: super::config::AI_REVIEW_TEMPERATURE,
        max_tokens: super::config::AI_REVIEW_MAX_TOKENS,
        stream: false,
    };

    let response = CLIENT
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(crate::data::ai::AI_API_KEY.as_str())
        .json(&request)
        .send()
        .instrument(tracing::info_span!("llm_request", category = "llm"))
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("DeepSeek API error ({status}): {body}").into());
    }

    let chat_response: ChatResponse = response.json().await?;
    let msg = chat_response
        .choices
        .into_iter()
        .next()
        .ok_or("DeepSeek API returned no choices")?
        .message;

    Ok(ChatResult {
        content: msg.content,
        tool_calls: msg.tool_calls,
    })
}
