use crate::prelude::*;

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
pub const AI_MAX_TOKENS: u32 = 150;
/// Sampling temperature for AI replies (0.0 = deterministic, higher = more random).
pub const AI_TEMPERATURE: f32 = 0.7;

// ── Channel lock (was DashSet AiChannelCache) ───────────────────────────────

const AI_CHANNEL_LOCK_TTL: u64 = 30;

/// Try to acquire the per-channel processing lock via Redis. Returns a guard
/// if acquired, or `None` if another request is already processing this
/// channel. When Redis is unavailable, returns a no-op guard.
pub async fn try_acquire_channel_lock(
    channel_id: u64,
) -> Option<crate::data::cache::RedisLockGuard> {
    let Some(mut conn) = crate::data::cache::conn().await else {
        // No Redis: return no-op guard (empty key).
        return Some(crate::data::cache::RedisLockGuard::new(
            String::new(),
            String::new(),
        ));
    };
    let key = format!("ai:ch_lock:{channel_id}");
    let token = format!("{}-{}", std::process::id(), rand::random::<u64>());
    if crate::data::cache::try_acquire_lock(&mut conn, &key, &token, AI_CHANNEL_LOCK_TTL).await {
        Some(crate::data::cache::RedisLockGuard::new(key, token))
    } else {
        None
    }
}

// ── Rate limiter (was moka Cache) ───────────────────────────────────────────

pub const AI_RATE_LIMIT_SECS: u64 = 10;

/// Check whether a user is rate-limited for AI prompts. Returns `true` if
/// rate-limited (should be blocked), `false` if allowed. When Redis is
/// unavailable, never rate-limits (single-instance fallback).
pub async fn check_ai_rate_limit(user_id: u64) -> bool {
    let Some(mut conn) = crate::data::cache::conn().await else {
        return false;
    };
    crate::data::cache::check_rate_limit(
        &mut conn,
        &format!("ai:rl:{user_id}"),
        AI_RATE_LIMIT_SECS,
    )
    .await
    .unwrap_or(false)
}
