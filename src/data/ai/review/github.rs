use std::{
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use moka::future::Cache;
use serde::Deserialize;

use crate::prelude::Error;

use super::config::{
    GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY,
    GITHUB_OAUTH_CLIENT_ID, GITHUB_OAUTH_SCOPE, GITHUB_TOKEN_TTL_SECS,
};

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If it fails it should do so the moment the app starts with [`LazyLock::force`] which is the intended behaviour."
    )]
    reqwest::Client::builder()
        .user_agent("serenity-discord-bot")
        .build()
        .expect("failed to build GitHub HTTP client")
});

pub static GITHUB_TOKEN_CACHE: LazyLock<Cache<u64, String>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(*GITHUB_TOKEN_TTL_SECS))
        .build()
});

// ── Device flow types ────────────────────────────────────────────────────────

pub struct DeviceCode {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Start the GitHub OAuth device flow. Returns codes the user needs to authorize.
pub async fn start_device_flow() -> Result<DeviceCode, Error> {
    let response = HTTP
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": GITHUB_OAUTH_CLIENT_ID.as_str(),
            "scope": GITHUB_OAUTH_SCOPE.as_str(),
        }))
        .send()
        .await?;

    // GitHub puts the diagnosis in the body (e.g. device_flow_disabled);
    // error_for_status would throw it away.
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GitHub device flow error ({status}): {body}").into());
    }

    let resp = response.json::<DeviceCodeResponse>().await?;

    Ok(DeviceCode {
        user_code: resp.user_code,
        verification_uri: resp.verification_uri,
        device_code: resp.device_code,
        expires_in: resp.expires_in,
        interval: resp.interval,
    })
}

/// Poll GitHub until the user authorizes or the code expires. Returns the access token.
pub async fn poll_device_flow(dc: &DeviceCode) -> Result<String, Error> {
    let mut interval = dc.interval;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(dc.expires_in);

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        if tokio::time::Instant::now() >= deadline {
            return Err("device flow authorization timed out".into());
        }

        let response = HTTP
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "client_id": GITHUB_OAUTH_CLIENT_ID.as_str(),
                "device_code": dc.device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("GitHub token poll error ({status}): {body}").into());
        }

        let resp = response.json::<TokenResponse>().await?;

        if let Some(token) = resp.access_token
            && !token.is_empty()
        {
            return Ok(token);
        }

        match resp.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                interval += 5;
                continue;
            }
            Some("expired_token") => return Err("device flow code expired".into()),
            Some("access_denied") => return Err("user denied the device flow authorization".into()),
            Some(other) => return Err(format!("device flow error: {other}").into()),
            None => continue,
        }
    }
}

/// Fetch the GitHub login name for the given token.
pub async fn fetch_login(token: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    struct UserResponse {
        login: String,
    }

    let resp = HTTP
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<UserResponse>()
        .await?;

    Ok(resp.login)
}

/// Returns true if the token has push, maintain, or admin permission on the repo.
pub async fn has_push_permission(token: &str, owner: &str, repo: &str) -> Result<bool, Error> {
    #[derive(Deserialize)]
    struct Permissions {
        #[serde(default)]
        admin: bool,
        #[serde(default)]
        maintain: bool,
        #[serde(default)]
        push: bool,
    }
    #[derive(Deserialize)]
    struct RepoResponse {
        permissions: Option<Permissions>,
    }

    let response = HTTP
        .get(format!("https://api.github.com/repos/{owner}/{repo}"))
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(false);
    }

    let resp = response.error_for_status()?.json::<RepoResponse>().await?;

    Ok(resp
        .permissions
        .map(|p| p.admin || p.maintain || p.push)
        .unwrap_or(false))
}

// ── GitHub App installation token ───────────────────────────────────────────

/// Generate a short-lived GitHub App installation access token for the given
/// repo owner. Discovers the installation ID dynamically by querying the
/// user and org endpoints — no per-user config needed.
pub async fn get_installation_token(owner: &str) -> Result<String, Error> {
    let app_id = GITHUB_APP_ID.as_str();
    let private_key_pem = GITHUB_APP_PRIVATE_KEY.as_str();

    // Sign a JWT for the app.
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?;
    let claims = serde_json::json!({
        "iat": now.as_secs(),
        "exp": now.as_secs() + 600,
        "iss": app_id,
    });
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| format!("invalid GITHUB_APP_PRIVATE_KEY: {e}"))?;
    let jwt = jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| format!("JWT signing failed: {e}"))?;

    // Discover the installation ID for this owner.
    let installation_id = discover_installation(owner, &jwt).await?;

    // Exchange the JWT for an installation access token.
    let response = HTTP
        .post(format!(
            "https://api.github.com/app/installations/{installation_id}/access_tokens"
        ))
        .header("Authorization", format!("Bearer {jwt}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "installation token error ({status}): {body}"
        )
        .into());
    }

    #[derive(Deserialize)]
    struct InstallationToken {
        token: String,
    }

    let token = response
        .json::<InstallationToken>()
        .await
        .map_err(|e| format!("failed to parse installation token: {e}"))?
        .token;

    Ok(token)
}

/// Try the user endpoint first, then the org endpoint, to find the GitHub
/// App installation ID for the given account name.
async fn discover_installation(owner: &str, jwt: &str) -> Result<String, Error> {
    let auth_header = format!("Bearer {jwt}");

    // Try user account first.
    let resp = HTTP
        .get(format!("https://api.github.com/users/{owner}/installation"))
        .header("Authorization", &auth_header)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if resp.status().is_success() {
        #[derive(Deserialize)]
        struct Installation {
            id: u64,
        }
        let id = resp
            .json::<Installation>()
            .await
            .map_err(|e| format!("failed to parse installation: {e}"))?
            .id;
        return Ok(id.to_string());
    }

    if resp.status() != reqwest::StatusCode::NOT_FOUND {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("installation lookup error ({status}): {body}").into());
    }

    // Try org account.
    let resp = HTTP
        .get(format!("https://api.github.com/orgs/{owner}/installation"))
        .header("Authorization", &auth_header)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if resp.status().is_success() {
        #[derive(Deserialize)]
        struct Installation {
            id: u64,
        }
        let id = resp
            .json::<Installation>()
            .await
            .map_err(|e| format!("failed to parse installation: {e}"))?
            .id;
        return Ok(id.to_string());
    }

    let body = resp.text().await.unwrap_or_default();
    Err(format!(
        "GitHub App is not installed for `{owner}` (tried user and org): {body}. \
         Install the app at https://github.com/apps/hu-tao-reviewer/installations/new"
    )
    .into())
}
