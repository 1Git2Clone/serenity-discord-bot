use crate::{data::config::CONFIG, prelude::*};
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

#[allow(unused, reason = "Follow the exact response API.")]
#[derive(Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message: AiMessage,
    pub done: bool,
    pub done_reason: String,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u32,
    pub prompt_eval_duration: u64,
    pub eval_count: u32,
    pub eval_duration: u64,
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
            num_predict: CONFIG.ai.num_predict,
            temperature: CONFIG.ai.temperature,
        }
    }
}

impl<'a> OllamaRequest<'a> {
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
            CONFIG.ai.model.as_str(),
            messages,
            CONFIG.ai.default_stream,
            OllamaOptions::default(),
        )
    }

    pub async fn call(&self, client: &Client) -> Result<String, Error> {
        let response = client
            .post(CONFIG.ai.chat_endpoint.as_str())
            .json(&self)
            .send()
            .await?;

        let text = response.text().await?;
        tracing::info!("Raw Ollama response: {}", text);

        match serde_json::from_str::<OllamaResponse>(&text) {
            Ok(parsed) => Ok(parsed.message.content),
            Err(why) => {
                let error_msg = format!("AI Call request failed! {why}\nRaw response: {}", text);
                tracing::error!(error_msg);
                Err(error_msg.into())
            }
        }
    }
}
