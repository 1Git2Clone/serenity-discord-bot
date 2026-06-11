use std::sync::LazyLock;

use crate::data::ai::config::AiChannelCache;

#[allow(
    clippy::expect_used,
    reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
)]
pub static GITHUB_OAUTH_CLIENT_ID: LazyLock<String> = LazyLock::new(|| {
    std::env::var("GITHUB_OAUTH_CLIENT_ID")
        .expect("Set the `GITHUB_OAUTH_CLIENT_ID` variable for /ai-review.")
});

pub static GITHUB_APP_ID: LazyLock<String> = LazyLock::new(|| {
    std::env::var("GITHUB_APP_ID")
        .expect("Set the `GITHUB_APP_ID` variable for /ai-review.")
});

pub static GITHUB_APP_PRIVATE_KEY: LazyLock<String> = LazyLock::new(|| {
    let path = std::env::var("GITHUB_APP_PRIVATE_KEY_PATH")
        .expect("Set the `GITHUB_APP_PRIVATE_KEY_PATH` variable for /ai-review.");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read `GITHUB_APP_PRIVATE_KEY_PATH` file: {path}"))
});

pub static GITHUB_OAUTH_SCOPE: LazyLock<String> = LazyLock::new(|| {
    match std::env::var("GITHUB_OAUTH_SCOPE") {
        Ok(var) => var,
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`GITHUB_OAUTH_SCOPE` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => "public_repo".to_string(),
    }
});

pub static GITHUB_TOKEN_TTL_SECS: LazyLock<u64> = LazyLock::new(|| {
    match std::env::var("GITHUB_TOKEN_TTL_SECS") {
        Ok(var) =>
        {
            #[allow(
                clippy::expect_used,
                reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
            )]
            var.parse::<u64>()
                .expect("`GITHUB_TOKEN_TTL_SECS` must be a valid u64.")
        }
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`GITHUB_TOKEN_TTL_SECS` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => 3600,
    }
});

pub static AI_REVIEW_MAX_ITERATIONS: LazyLock<u32> = LazyLock::new(|| {
    match std::env::var("AI_REVIEW_MAX_ITERATIONS") {
        Ok(var) =>
        {
            #[allow(
                clippy::expect_used,
                reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
            )]
            var.parse::<u32>()
                .expect("`AI_REVIEW_MAX_ITERATIONS` must be a valid u32.")
        }
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`AI_REVIEW_MAX_ITERATIONS` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => 20,
    }
});

pub static AI_REVIEW_TIMEOUT_SECS: LazyLock<u64> = LazyLock::new(|| {
    match std::env::var("AI_REVIEW_TIMEOUT_SECS") {
        Ok(var) =>
        {
            #[allow(
                clippy::expect_used,
                reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
            )]
            var.parse::<u64>()
                .expect("`AI_REVIEW_TIMEOUT_SECS` must be a valid u64.")
        }
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`AI_REVIEW_TIMEOUT_SECS` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => 600,
    }
});

/// Upper bound on tokens generated per review turn.
pub const AI_REVIEW_MAX_TOKENS: u32 = 4096;
/// Sampling temperature for review turns.
pub const AI_REVIEW_TEMPERATURE: f32 = 0.3;
/// Maximum bytes per tool result before truncation.
pub const TOOL_OUTPUT_LIMIT: usize = 64 * 1024;

/// Global guard — one review at a time. Uses a sentinel key (0) since only
/// one review can run concurrently regardless of target PR.
pub static AI_REVIEW_GUARD: LazyLock<AiChannelCache> = LazyLock::new(AiChannelCache::new);
