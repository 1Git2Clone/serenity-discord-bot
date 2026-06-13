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
        .expect("Failed to build reqwest client for the AI tool loop")
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

/// Send a tool-calling chat-completions request to DeepSeek. Returns the
/// model's content and/or tool calls.
///
/// This bypasses the `llm` crate on purpose: as of `llm` 1.3.8 the DeepSeek
/// backend's `chat_with_tools` is unimplemented (`todo!()`), so tool loops talk
/// to the `/chat/completions` endpoint directly. `temperature` and `max_tokens`
/// are passed in rather than read from a global so different callers (review,
/// chat tools) can tune them independently.
#[tracing::instrument(skip(messages, tools), fields(category = "llm"))]
pub async fn chat(
    messages: &[Message],
    tools: &[Tool],
    temperature: f32,
    max_tokens: u32,
) -> Result<ChatResult, Box<dyn std::error::Error + Send + Sync>> {
    let request = ChatRequest {
        model: crate::data::ai::DEFAULT_MODEL.to_string(),
        messages: messages.to_vec(),
        tools: tools.to_vec(),
        temperature,
        max_tokens,
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

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), serde_json::Error>;

    #[test]
    fn system_and_user_messages_tag_their_role() -> TestResult {
        assert_eq!(
            serde_json::to_value(Message::System {
                content: "be brief".into()
            })?,
            serde_json::json!({"role": "system", "content": "be brief"})
        );
        assert_eq!(
            serde_json::to_value(Message::User {
                content: "hello".into()
            })?,
            serde_json::json!({"role": "user", "content": "hello"})
        );
        Ok(())
    }

    #[test]
    fn tool_result_message_carries_the_call_id() -> TestResult {
        assert_eq!(
            serde_json::to_value(Message::Tool {
                tool_call_id: "call_1".into(),
                content: "a.txt".into(),
            })?,
            serde_json::json!({
                "role": "tool",
                "tool_call_id": "call_1",
                "content": "a.txt"
            })
        );
        Ok(())
    }

    #[test]
    fn assistant_tool_call_round_trips_through_the_wire_shape() -> TestResult {
        let call = ToolCall {
            id: "call_1".into(),
            call_type: "function".into(),
            function: ToolCallFunction {
                name: "git_diff".into(),
                arguments: r#"{"path":"a.txt"}"#.into(),
            },
        };
        assert_eq!(
            serde_json::to_value(Message::Assistant {
                content: None,
                tool_calls: Some(vec![call]),
            })?,
            serde_json::json!({
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "git_diff",
                        "arguments": r#"{"path":"a.txt"}"#
                    }
                }]
            })
        );
        Ok(())
    }

    #[test]
    fn assistant_without_tool_calls_omits_the_field() -> TestResult {
        // `skip_serializing_if` keeps a plain assistant turn from carrying a
        // null `tool_calls`, which the API rejects.
        let json = serde_json::to_value(Message::Assistant {
            content: Some("all peaceful".into()),
            tool_calls: None,
        })?;
        assert_eq!(
            json,
            serde_json::json!({"role": "assistant", "content": "all peaceful"})
        );
        assert!(json.get("tool_calls").is_none());
        Ok(())
    }

    #[test]
    fn tool_call_deserializes_from_an_api_response() -> TestResult {
        // The shape DeepSeek returns inside `choices[].message.tool_calls`.
        let raw = serde_json::json!({
            "id": "call_9",
            "type": "function",
            "function": {"name": "read_file", "arguments": "{}"}
        });
        let call: ToolCall = serde_json::from_value(raw)?;
        assert_eq!(call.id, "call_9");
        assert_eq!(call.function.name, "read_file");
        Ok(())
    }
}
