use std::{sync::LazyLock, time::Duration};

use crate::{enums::schemas::AiChannelsTable, prelude::*};
use ::serenity::all::GetMessages;
use dashmap::DashSet;
use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use moka::future::Cache;

// Backend is chosen at compile time via the `ai-<backend>` Cargo feature. Fail
// loudly if `ai` is on but no backend was picked (e.g. `--features ai` alone),
// instead of letting `LLMBuilder::build()` blow up at runtime.
#[cfg(not(any(
    feature = "ai-deepseek",
    feature = "ai-ollama",
    feature = "ai-anthropic",
    feature = "ai-openai",
    feature = "ai-google",
    feature = "ai-groq",
)))]
compile_error!(
    "The `ai` feature needs a backend. Enable exactly one of: \
     `ai-deepseek`, `ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, `ai-groq`."
);

#[cfg(feature = "ai-ollama")]
pub static CHAT_ENDPOINT: LazyLock<Option<String>> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_CHAT_ENDPOINT").ok()
});
pub static DEFAULT_MODEL: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_MODEL").expect("Set the `AI_MODEL` variable.")
});
#[cfg(any(
    feature = "ai-anthropic",
    feature = "ai-deepseek",
    feature = "ai-openai",
    feature = "ai-google",
    feature = "ai-groq",
))]
pub static AI_API_KEY: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_API_KEY")
        .expect("Set the `AI_API_KEY` variable when using any of the following features: `ai-anthropic`, `ai-openai`, `ai-deepseek`, `ai-google`, or `ai-groq`.")
});
/// Defaults to 10 if not present like it was in:
/// - https://github.com/1Git2Clone/serenity-discord-bot/commit/a7d2a8c157eb966335c1dcc9a3995bc48b8aa193
///
/// Pretty low if you're using paid APIs or have a system with over 64GB of RAM running a local
/// model.
pub static AI_MAX_MSG_CONTEXT: LazyLock<u32> = LazyLock::new(|| {
    match std::env::var("AI_MAX_MSG_CONTEXT") {
        Ok(var) =>
        {
            #[allow(
                clippy::expect_used,
                reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
            )]
            var.parse::<u32>()
                .expect("`AI_MAX_MSG_CONTEXT` Must be a valid unsigned 32 bit integer.")
        }
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`AI_MAX_MSG_CONTEXT` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => 10,
    }
});

/// Upper bound on tokens generated per AI reply
const AI_MAX_TOKENS: u32 = 150;
/// Sampling temperature for AI replies (0.0 = deterministic, higher = more random).
const AI_TEMPERATURE: f32 = 0.7;

pub struct AiChannelCache {
    inner: DashSet<u64>,
}

impl AiChannelCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_acquire(&self, key: u64) -> Option<AiCacheGuard<'_>> {
        if !self.inner.insert(key) {
            return None;
        }

        Some(AiCacheGuard { key, cache: self })
    }
}

impl Default for AiChannelCache {
    fn default() -> Self {
        Self {
            inner: DashSet::new(),
        }
    }
}

pub struct AiCacheGuard<'a> {
    key: u64,
    cache: &'a AiChannelCache,
}

impl Drop for AiCacheGuard<'_> {
    fn drop(&mut self) {
        self.cache.inner.remove(&self.key);
    }
}

pub static AI_CHANNEL_CACHE: LazyLock<AiChannelCache> = LazyLock::new(AiChannelCache::new);
pub const AI_RATE_LIMIT_SECS: u64 = 10;
pub static AI_RATE_LIMIT: LazyLock<Cache<UserId, ()>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(AI_RATE_LIMIT_SECS))
        .build()
});

/// Channels where the bot auto-replies to every message. Backed by the
/// `ai_channels` table but kept in memory so the message handler avoids a DB hit
/// per message. Populated by [`init_registered_channels`] at startup.
pub static AI_REGISTERED_CHANNELS: LazyLock<DashSet<u64>> = LazyLock::new(DashSet::new);

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
/// feature. Exactly one is expected (enforced by the `compile_error!` guard at
/// the top of this module); each `return` below only exists when its feature is on.
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

/// Set on the builder, not sent per call — the crate's `ChatRole` has no system
/// variant.
const SYSTEM_PROMPT: &str = r#"
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

/// Built once so the backend's connection pool is reused instead of
/// re-handshaking TLS on every call.
pub static AI_PROVIDER: LazyLock<Box<dyn LLMProvider>> = LazyLock::new(|| {
    let mut builder = LLMBuilder::new()
        .backend(ai_backend())
        .model(DEFAULT_MODEL.as_str())
        .system(SYSTEM_PROMPT)
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
});

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

    let response = AI_PROVIDER.chat(&conversation).await?;
    tracing::info!("Raw AI response: {response}");

    Ok(response.text().unwrap_or_else(|| response.to_string()))
}

/// Load the registered AI channels from the DB into the in-memory set.
pub async fn init_registered_channels(pool: &PgPool) -> Result<(), Error> {
    for channel_id in AiChannelsTable::fetch_all(pool).await? {
        AI_REGISTERED_CHANNELS.insert(channel_id as u64);
    }
    Ok(())
}

pub fn is_ai_channel(channel_id: u64) -> bool {
    AI_REGISTERED_CHANNELS.contains(&channel_id)
}

/// Toggle a channel's AI registration in both the DB and the in-memory set.
/// Returns `true` if it's now registered, `false` if it was removed.
pub async fn toggle_ai_channel(pool: &PgPool, channel_id: u64, guild_id: u64) -> Result<bool, Error> {
    if AI_REGISTERED_CHANNELS.contains(&channel_id) {
        AiChannelsTable::unregister(pool, channel_id as i64).await?;
        AI_REGISTERED_CHANNELS.remove(&channel_id);
        Ok(false)
    } else {
        AiChannelsTable::register(pool, channel_id as i64, guild_id as i64).await?;
        AI_REGISTERED_CHANNELS.insert(channel_id);
        Ok(true)
    }
}

/// Build a prompt from prior channel messages plus the current one. Empty
/// messages are skipped; the bot's own messages map to the assistant role.
pub fn messages_to_prompt(
    previous_messages: &[serenity::Message],
    bot_user_id: u64,
    current_message: &str,
) -> Vec<AiMessage> {
    let mut res = Vec::with_capacity(previous_messages.len() + 1);

    for m in previous_messages {
        if m.content.trim().is_empty() {
            continue;
        }
        res.push(AiMessage::new(
            if m.author.id.get() == bot_user_id {
                "assistant"
            } else {
                "user"
            },
            &m.content,
        ));
    }

    res.push(AiMessage::new("user", current_message));

    res
}

/// Reply to a message in a registered AI channel, honoring the per-user rate
/// limit and the per-channel processing lock.
#[tracing::instrument(
    skip(ctx, data, new_message),
    fields(
        category = "ai_auto_reply",
        author = %new_message.author.id,
        channel_id = %new_message.channel_id,
    )
)]
pub async fn handle_ai_channel_message(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &serenity::Message,
) -> Result<(), Error> {
    if !is_ai_channel(new_message.channel_id.get()) {
        return Ok(());
    }

    if AI_RATE_LIMIT.get(&new_message.author.id).await.is_some() {
        return Ok(());
    }

    let Some(_guard) = AI_CHANNEL_CACHE.try_acquire(new_message.channel_id.get()) else {
        return Ok(());
    };
    AI_RATE_LIMIT.insert(new_message.author.id, ()).await;

    let _ = new_message.channel_id.broadcast_typing(ctx).await;

    let mut previous = new_message
        .channel_id
        .messages(
            ctx,
            GetMessages::new()
                .before(new_message.id)
                .limit((*AI_MAX_MSG_CONTEXT).min(100) as u8),
        )
        .await
        .unwrap_or_default();
    previous.retain(|m| !m.author.bot || m.author.id.get() == data.bot_user.id.get());
    previous.reverse();

    let prompt = messages_to_prompt(&previous, data.bot_user.id.get(), &new_message.content);
    let response = chat(&prompt).await?;

    new_message.reply(ctx, response).await?;

    Ok(())
}
