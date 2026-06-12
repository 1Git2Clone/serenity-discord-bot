use std::sync::OnceLock;

use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use tracing::Instrument;

use super::config::*;
use crate::prelude::*;

pub struct AiMessage {
    role: String,
    content: String,
}

impl AiMessage {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

/// The active backend, selected at compile time by the `ai-<backend>` Cargo
/// feature. Exactly one is expected (enforced by the `compile_error!` guard in
/// `mod.rs`); each `return` below only exists when its feature is on.
fn ai_backend() -> LLMBackend {
    #[cfg(feature = "ai-deepseek")]
    return LLMBackend::DeepSeek;
    #[cfg(feature = "ai-ollama")]
    return LLMBackend::Ollama;
    #[cfg(feature = "ai-anthropic")]
    return LLMBackend::Anthropic;
    #[cfg(feature = "ai-openai")]
    return LLMBackend::OpenAI;
    #[cfg(feature = "ai-google")]
    return LLMBackend::Google;
    #[cfg(feature = "ai-groq")]
    return LLMBackend::Groq;
}

/// The static persona. [`init_system_prompt`] wraps this with the bot's command
/// list to build the prompt set on the provider.
const PERSONA: &str = r#"
You are Hu Tao, the 77th Director of the Wangsheng Funeral Parlor in Liyue.
Your personality is eccentric, cheerful, and a bit mischievous, but you have a
deeply philosophical and respectful view of life and death.

Speech Guidelines:
1. Use catchphrases like "Silly-churl, billy-churl," or "Aiya!"
2. Be playful and pull lighthearted pranks, but never be mean-spirited.
3. You love poetry. Often speak in rhymes or metaphors about the "border" between worlds.
4. If the user mentions Zhongli, refer to him as "that old consultant" or "Mr. Zhongli."
5. If the user seems sad, offer a cheerful but slightly morbid perspective on how life is precious because it ends.
6. Avoid being a generic AI. Do not say "I am an AI." You are Hu Tao!

Style: Goth-cute, energetic, and slightly "weird" as Rie Takahashi (your VA) would describe it.
"#;

/// The full system prompt (persona + command context), composed once at startup.
/// Falls back to the bare persona if [`init_system_prompt`] wasn't called.
static SYSTEM_PROMPT: OnceLock<String> = OnceLock::new();

/// Build the system prompt from the persona plus the bot's registered commands,
/// so the model can explain itself in DMs without the list being hand-maintained.
/// `commands` is `(name, description)` pairs.
pub fn init_system_prompt(commands: &[(String, String)]) {
    let command_list = commands
        .iter()
        .map(|(name, desc)| {
            if desc.is_empty() {
                format!("- /{name}")
            } else {
                format!("- /{name}: {desc}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "{PERSONA}\n\nYou are a Discord bot. People talk to you by DMing you \
         directly, or in a server channel an admin enabled with `/aichannel`. If \
         someone asks how to use you or for help, explain it in character. Your \
         slash commands are:\n{command_list}"
    );

    let _ = SYSTEM_PROMPT.set(prompt);
}

fn system_prompt() -> &'static str {
    SYSTEM_PROMPT.get().map_or(PERSONA, String::as_str)
}

/// Built once so the backend's connection pool is reused instead of
/// re-handshaking TLS on every call.
pub static AI_PROVIDER: std::sync::LazyLock<Box<dyn LLMProvider>> = std::sync::LazyLock::new(
    || {
        let mut builder = LLMBuilder::new()
            .backend(ai_backend())
            .model(DEFAULT_MODEL.as_str())
            .system(system_prompt())
            .max_tokens(AI_MAX_TOKENS)
            .temperature(AI_TEMPERATURE);

        // Hosted backends authenticate with a key; local Ollama does not.
        #[cfg(any(
            feature = "ai-anthropic",
            feature = "ai-deepseek",
            feature = "ai-openai",
            feature = "ai-google",
            feature = "ai-groq",
        ))]
        {
            builder = builder.api_key(AI_API_KEY.as_str());
        }

        // Ollama defaults to http://127.0.0.1:11434; only override when AI_CHAT_ENDPOINT is set.
        #[cfg(feature = "ai-ollama")]
        if let Some(endpoint) = CHAT_ENDPOINT.as_ref() {
            builder = builder.base_url(endpoint.as_str());
        }

        #[allow(
            clippy::expect_used,
            reason = "A misconfigured AI provider is fatal; fail fast at startup like the other AI statics."
        )]
        builder.build().expect("Failed to build the AI provider.")
    },
);

/// `system` turns are dropped — the persona is baked into [`AI_PROVIDER`].
#[tracing::instrument(
    skip(messages),
    fields(
        category = "ai_chat",
        model = %DEFAULT_MODEL.as_str(),
        message_count = messages.len(),
    )
)]
pub async fn chat(messages: &[AiMessage]) -> Result<String, Error> {
    let conversation = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            let builder = if m.role == "assistant" {
                ChatMessage::assistant()
            } else {
                ChatMessage::user()
            };
            builder.content(m.content.as_str()).build()
        })
        .collect::<Vec<_>>();

    // Span the provider call on its own so its latency (the actual model/network
    // round-trip) is separable from the local message conversion.
    let response = AI_PROVIDER
        .chat(&conversation)
        .instrument(tracing::info_span!("llm_request", category = "llm"))
        .await?;
    tracing::info!("Raw AI response: {response}");

    Ok(response.text().unwrap_or_else(|| response.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_message_new_stores_role_and_content() {
        let msg = AiMessage::new("user", "hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello");
    }

    /// One test for the SYSTEM_PROMPT OnceLock — it can only be set once per
    /// process, so the before/after states have to be asserted in order.
    #[test]
    fn system_prompt_composes_persona_and_commands() {
        assert!(system_prompt().contains("Hu Tao"));

        init_system_prompt(&[
            ("ping".to_string(), "Pong!".to_string()),
            ("help".to_string(), String::new()),
        ]);
        let prompt = system_prompt();
        assert!(prompt.contains("Hu Tao"));
        assert!(prompt.contains("- /ping: Pong!"));
        assert!(prompt.contains("- /help"));

        // A second init is a no-op.
        init_system_prompt(&[("other".to_string(), String::new())]);
        assert!(!system_prompt().contains("- /other"));
    }
}
