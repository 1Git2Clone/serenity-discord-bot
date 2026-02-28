use std::sync::LazyLock;

use crate::prelude::*;
use dashmap::DashSet;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AiChannelCache {
    inner: DashSet<u64>,
}

impl AiChannelCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_acquire(&self, key: u64) -> Option<AiCacheGuard<'_>> {
        if self.inner.contains(&key) {
            return None;
        }

        self.inner.insert(key);
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
    pub const DEFAULT_MODEL: &'static str = "qwen2.5:1.5b";
    pub const DEFAULT_STREAM: bool = false;
    pub const CHAT_ENDPOINT: &'static str = "http://localhost:11434/api/chat";

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
            Self::DEFAULT_MODEL,
            messages,
            Self::DEFAULT_STREAM,
            OllamaOptions::default(),
        )
    }

    pub async fn call(&self, client: &Client) -> Result<String, Error> {
        match client
            .post(Self::CHAT_ENDPOINT)
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
