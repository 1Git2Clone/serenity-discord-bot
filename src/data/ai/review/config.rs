use std::sync::LazyLock;

#[allow(
    clippy::expect_used,
    reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
)]
pub static GITHUB_OAUTH_CLIENT_ID: LazyLock<String> = LazyLock::new(|| {
    std::env::var("GITHUB_OAUTH_CLIENT_ID")
        .expect("Set the `GITHUB_OAUTH_CLIENT_ID` variable for /ai-review.")
});

pub static GITHUB_APP_ID: LazyLock<String> = LazyLock::new(|| {
    std::env::var("GITHUB_APP_ID").expect("Set the `GITHUB_APP_ID` variable for /ai-review.")
});

pub static GITHUB_APP_PRIVATE_KEY: LazyLock<String> = LazyLock::new(|| {
    let path = std::env::var("GITHUB_APP_PRIVATE_KEY_PATH")
        .expect("Set the `GITHUB_APP_PRIVATE_KEY_PATH` variable for /ai-review.");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read `GITHUB_APP_PRIVATE_KEY_PATH` file: {path}"))
});

pub static GITHUB_OAUTH_SCOPE: LazyLock<String> =
    LazyLock::new(|| match std::env::var("GITHUB_OAUTH_SCOPE") {
        Ok(var) => var,
        Err(std::env::VarError::NotUnicode(var)) => {
            panic!("`GITHUB_OAUTH_SCOPE` environment variable is not valid unicode. Var: {var:?}")
        }
        Err(std::env::VarError::NotPresent) => "public_repo".to_string(),
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
            panic!(
                "`GITHUB_TOKEN_TTL_SECS` environment variable is not valid unicode. Var: {var:?}"
            )
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
            panic!(
                "`AI_REVIEW_MAX_ITERATIONS` environment variable is not valid unicode. Var: {var:?}"
            )
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
            panic!(
                "`AI_REVIEW_TIMEOUT_SECS` environment variable is not valid unicode. Var: {var:?}"
            )
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

// ── Global review guard (was DashSet AiChannelCache) ────────────────────────

const AI_REVIEW_GUARD_TTL: u64 = 600;

/// Try to acquire the global AI review guard via Redis. Returns a guard if
/// acquired, or `None` if a review is already running. When Redis is
/// unavailable, returns a no-op guard.
pub async fn try_acquire_review_guard() -> Option<crate::data::cache::RedisLockGuard> {
    let Some(mut conn) = crate::data::cache::conn().await else {
        // No Redis: return no-op guard (empty key).
        return Some(crate::data::cache::RedisLockGuard::new(
            String::new(),
            String::new(),
        ));
    };
    let key = "ai:review_guard".to_string();
    let token = format!("{}-{}", std::process::id(), rand::random::<u64>());
    if crate::data::cache::try_acquire_lock(&mut conn, &key, &token, AI_REVIEW_GUARD_TTL).await {
        Some(crate::data::cache::RedisLockGuard::new(key, token))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Only the statics with fallback defaults are forced here — the required
    // ones (client ID, app ID, private key) panic without their env vars.
    // Assertions are loose because the env may override the defaults.
    #[test]
    fn defaultable_statics_resolve() {
        assert!(!GITHUB_OAUTH_SCOPE.is_empty());
        assert!(*GITHUB_TOKEN_TTL_SECS > 0);
        assert!(*AI_REVIEW_MAX_ITERATIONS > 0);
        assert!(*AI_REVIEW_TIMEOUT_SECS > 0);
    }
}
