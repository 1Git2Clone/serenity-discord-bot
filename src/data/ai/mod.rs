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

mod channels;
mod config;
mod context;
mod handler;
mod provider;
pub mod review;
pub(crate) mod tools;

pub use channels::{init_registered_channels, toggle_ai_channel};
pub use config::{
    AI_MAX_MSG_CONTEXT, AI_RATE_LIMIT_SECS, DEFAULT_MODEL, check_ai_rate_limit,
    try_acquire_channel_lock,
};
pub use context::{author_name, channel_context, record_message};
pub use handler::handle_ai_channel_message;
pub use provider::{AI_PROVIDER, chat, init_system_prompt};

#[cfg(any(
    feature = "ai-anthropic",
    feature = "ai-deepseek",
    feature = "ai-openai",
    feature = "ai-google",
    feature = "ai-groq",
))]
pub use config::AI_API_KEY;
