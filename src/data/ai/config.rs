use std::{sync::LazyLock, time::Duration};

use crate::prelude::*;
use dashmap::DashSet;
use moka::future::Cache;

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
