use std::sync::LazyLock;

use crate::data::ai::config::AiChannelCache;

pub static GITHUB_APP_TOKEN: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("GITHUB_APP_TOKEN")
        .expect("Set the `GITHUB_APP_TOKEN` variable for /ai-review.")
});

pub static AI_REVIEW_ROLE: LazyLock<u64> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    std::env::var("AI_REVIEW_ROLE")
        .expect("Set the `AI_REVIEW_ROLE` variable (role ID allowed to use /ai-review).")
        .parse::<u64>()
        .expect("`AI_REVIEW_ROLE` must be a valid u64.")
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
