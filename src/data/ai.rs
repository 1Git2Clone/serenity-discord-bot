use std::{sync::LazyLock, time::Duration};

use crate::prelude::*;
use dashmap::DashSet;
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub static CHAT_ENDPOINT: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_CHAT_ENDPOINT").expect("Set the `AI_CHAT_ENDPOINT` environment variable.")
});
pub static DEFAULT_MODEL: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_MODEL").expect("Set the `AI_MODEL` variable.")
});

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

#[derive(Serialize, Deserialize)]
pub struct AiMessage {
    role: String,
    content: String,
}

/// [Ollama documentation](https://docs.ollama.com/api/chat#body-options).
#[derive(Serialize)]
pub struct OllamaOptions {
    /// [Ollama documentation](https://docs.ollama.com/api/chat#body-options-num-predict).
    pub num_predict: u32,
    /// [Ollama documentation](https://docs.ollama.com/api/chat#body-options-temperature).
    pub temperature: f64,
}

/// [Ollama documentation](https://docs.ollama.com/api/chat).
#[derive(Serialize)]
pub struct OllamaRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [AiMessage],
    pub stream: bool,
    pub options: OllamaOptions,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    // pub model: String,
    // pub created_at: DateTime<chrono::Utc>,
    pub message: AiMessage,
    // pub done: bool,
    // pub done_reason: String,
    // pub total_duration: u64,
    // pub load_duration: u64,
    // pub prompt_eval_count: u32,
    // pub prompt_eval_duration: u64,
    // pub eval_count: u32,
    // pub eval_duration: u64,
}

impl AiMessage {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

impl Default for OllamaOptions {
    fn default() -> Self {
        Self {
            num_predict: 150,
            temperature: 0.7,
        }
    }
}

impl<'a> OllamaRequest<'a> {
    pub const DEFAULT_STREAM: bool = false;

    pub fn new(
        model: &'a str,
        messages: &'a [AiMessage],
        stream: bool,
        options: OllamaOptions,
    ) -> Self {
        Self {
            model,
            messages,
            stream,
            options,
        }
    }

    pub fn from(messages: &'a [AiMessage]) -> Self {
        Self::new(
            DEFAULT_MODEL.as_str(),
            messages,
            Self::DEFAULT_STREAM,
            OllamaOptions::default(),
        )
    }

    pub async fn call(&self, client: &Client) -> Result<String, Error> {
        match client
            .post(CHAT_ENDPOINT.as_str())
            .json(&self)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await
        {
            Ok(response) => Ok(response.message.content),
            Err(why) => {
                let error_msg = format!("AI Call request failed! {why}");
                tracing::info!(error_msg);
                Err(error_msg.into())
            }
        }
    }
}
