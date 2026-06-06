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
use redis::{AsyncCommands, aio::ConnectionManager};
use tokio::sync::OnceCell;

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

/// How long an idle context window lives in Redis before eviction.
const AI_CTX_TTL_SECS: i64 = 1800;
/// Separates the author id from the content in a stored entry. Unit Separator,
/// which won't appear in Discord message text.
const AI_CTX_SEP: char = '\u{1f}';

/// Shared Redis connection, or `None` when `REDIS_URL` is unset/unreachable (in
/// which case context falls back to fetching from Discord).
static AI_REDIS: OnceCell<Option<ConnectionManager>> = OnceCell::const_new();

async fn redis_conn() -> Option<ConnectionManager> {
    AI_REDIS
        .get_or_init(|| async {
            let url = std::env::var("REDIS_URL").ok()?;
            match redis::Client::open(url) {
                Ok(client) => match client.get_connection_manager().await {
                    Ok(conn) => {
                        tracing::info!("Connected to Redis for AI context caching.");
                        Some(conn)
                    }
                    Err(why) => {
                        tracing::warn!("Redis unavailable ({why}); using Discord fetches.");
                        None
                    }
                },
                Err(why) => {
                    tracing::warn!("Invalid REDIS_URL ({why}); using Discord fetches.");
                    None
                }
            }
        })
        .await
        .clone()
}

/// Connect to Redis at startup so the status is logged before the first message.
pub async fn init_redis() {
    let _ = redis_conn().await;
}

fn ctx_key(channel_id: u64) -> String {
    format!("ai:ctx:{channel_id}")
}

fn encode_entry(author_id: u64, content: &str) -> String {
    format!("{author_id}{AI_CTX_SEP}{content}")
}

fn entry_to_message(entry: &str, bot_user_id: u64) -> Option<AiMessage> {
    let (author_id, content) = entry.split_once(AI_CTX_SEP)?;
    if content.trim().is_empty() {
        return None;
    }
    let role = if author_id.parse::<u64>().ok()? == bot_user_id {
        "assistant"
    } else {
        "user"
    };
    Some(AiMessage::new(role, content))
}

/// Append a message to a channel's window, but only if the window already exists
/// (i.e. the channel is "warm" from a prior AI interaction). No-op without Redis.
pub async fn record_message(channel_id: u64, author_id: u64, content: &str) {
    let Some(mut conn) = redis_conn().await else {
        return;
    };

    let script = redis::Script::new(
        r"if redis.call('EXISTS', KEYS[1]) == 1 then
            redis.call('RPUSH', KEYS[1], ARGV[1])
            redis.call('LTRIM', KEYS[1], -tonumber(ARGV[2]), -1)
            redis.call('EXPIRE', KEYS[1], ARGV[3])
        end
        return 1",
    );

    let result: redis::RedisResult<i64> = script
        .key(ctx_key(channel_id))
        .arg(encode_entry(author_id, content))
        .arg(*AI_MAX_MSG_CONTEXT as i64)
        .arg(AI_CTX_TTL_SECS)
        .invoke_async(&mut conn)
        .await;

    if let Err(why) = result {
        tracing::warn!("Failed to record message in Redis: {why}");
    }
}

/// The recent conversation for a channel as a prompt.
///
/// Reads the Redis window; on a cold channel (or without Redis) it seeds the
/// window from a one-off Discord fetch. `current` is appended as a trailing user
/// turn — used by `/ai`, whose prompt isn't a channel message; the auto-reply
/// passes `None` since the triggering message is already in the window.
pub async fn channel_context(
    cache_http: impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
    current: Option<&str>,
) -> Vec<AiMessage> {
    let key = ctx_key(channel_id.get());

    let mut prompt: Vec<AiMessage> = match redis_conn().await {
        Some(mut conn) => {
            let entries: Vec<String> = conn.lrange(&key, 0, -1).await.unwrap_or_default();
            if entries.is_empty() {
                seed_from_discord(&cache_http, channel_id, bot_user_id, &key).await
            } else {
                entries
                    .iter()
                    .filter_map(|e| entry_to_message(e, bot_user_id))
                    .collect()
            }
        }
        None => fetch_from_discord(&cache_http, channel_id, bot_user_id).await,
    };

    if let Some(current) = current {
        prompt.push(AiMessage::new("user", current));
    }

    prompt
}

/// Fetch the recent window from Discord and map it to prompt messages.
async fn fetch_from_discord(
    cache_http: &impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
) -> Vec<AiMessage> {
    let limit = (*AI_MAX_MSG_CONTEXT).min(100) as u8;
    let mut messages = channel_id
        .messages(cache_http, GetMessages::new().limit(limit))
        .await
        .unwrap_or_default();
    messages.retain(|m| !m.author.bot || m.author.id.get() == bot_user_id);
    messages.reverse();

    messages
        .iter()
        .filter(|m| !m.content.trim().is_empty())
        .map(|m| {
            let role = if m.author.id.get() == bot_user_id {
                "assistant"
            } else {
                "user"
            };
            AiMessage::new(role, &m.content)
        })
        .collect()
}

/// Seed a cold channel's Redis window from Discord and return the prompt.
async fn seed_from_discord(
    cache_http: &impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
    key: &str,
) -> Vec<AiMessage> {
    let limit = (*AI_MAX_MSG_CONTEXT).min(100) as u8;
    let mut messages = channel_id
        .messages(cache_http, GetMessages::new().limit(limit))
        .await
        .unwrap_or_default();
    messages.retain(|m| {
        (!m.author.bot || m.author.id.get() == bot_user_id) && !m.content.trim().is_empty()
    });
    messages.reverse();

    if let Some(mut conn) = redis_conn().await
        && !messages.is_empty()
    {
        let encoded: Vec<String> = messages
            .iter()
            .map(|m| encode_entry(m.author.id.get(), &m.content))
            .collect();
        let result: redis::RedisResult<()> = redis::pipe()
            .atomic()
            .del(key)
            .ignore()
            .rpush(key, encoded)
            .ignore()
            .expire(key, AI_CTX_TTL_SECS)
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(why) = result {
            tracing::warn!("Failed to seed Redis window: {why}");
        }
    }

    messages
        .iter()
        .map(|m| {
            let role = if m.author.id.get() == bot_user_id {
                "assistant"
            } else {
                "user"
            };
            AiMessage::new(role, &m.content)
        })
        .collect()
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

    // The triggering message is already in the window (recorded in `handle_message`
    // before this runs), so no trailing turn is appended.
    let prompt = channel_context(ctx, new_message.channel_id, data.bot_user.id.get(), None).await;
    let response = chat(&prompt).await?;

    new_message.reply(ctx, response).await?;

    Ok(())
}
