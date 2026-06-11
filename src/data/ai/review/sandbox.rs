use std::{
    path::Path,
    process::Stdio,
};

use serde_json::Value;
use tokio::process::Command as TokioCommand;

use super::config::TOOL_OUTPUT_LIMIT;

// ── Workspace ───────────────────────────────────────────────────────────────

pub struct Workspace {
    dir: tempfile::TempDir,
    base_ref: String,
}

impl Workspace {
    /// Access the workspace directory path (for `current_dir` on subprocesses).
    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl Workspace {
    /// Clone `owner/repo`, checkout PR `pr`, and resolve the PR's base ref.
    #[tracing::instrument(
        skip(token),
        fields(category = "ai_review", owner = %owner, repo = %repo, pr = %pr)
    )]
    pub async fn checkout(
        owner: &str,
        repo: &str,
        pr: u64,
        token: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dir = tempfile::tempdir()?;

        // Clone (shallow).
        run_gh_checked(
            dir.path(),
            token,
            &[
                "repo",
                "clone",
                &format!("{owner}/{repo}"),
                ".",
                "--",
                "--depth=50",
            ],
        )
        .await?;

        // Fetch the PR head and check it out.
        // We avoid `gh pr checkout` because its internal `git checkout -b
        // --track` fails on shallow clones (Git 2.54+).
        run_git_checked(
            dir.path(),
            &["fetch", "origin", &format!("pull/{pr}/head")],
        )
        .await?;
        run_git_checked(dir.path(), &["checkout", "FETCH_HEAD"]).await?;

        // Resolve base ref name.
        let view_out = run_gh_stdout(
            dir.path(),
            token,
            &["pr", "view", &pr.to_string(), "--json", "baseRefName"],
        )
        .await?;
        let json: Value = serde_json::from_str(&view_out)?;
        let base_name = json["baseRefName"]
            .as_str()
            .ok_or("failed to parse baseRefName from `gh pr view` output")?;

        // Fetch the base ref.
        run_git_checked(dir.path(), &["fetch", "origin", base_name]).await?;

        Ok(Self {
            dir,
            base_ref: format!("origin/{base_name}"),
        })
    }

    // ── Tool execution ──────────────────────────────────────────────────

    /// Execute a named tool with JSON arguments. Errors are returned as
    /// strings so the agent loop can relay them to the model.
    #[tracing::instrument(
        skip(self, args),
        fields(category = "ai_review_tool", tool = %name)
    )]
    pub async fn execute(&self, name: &str, args: Value) -> String {
        match name {
            "list_files" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                run_git(self.dir.path(), &["ls-files", path]).await
            }
            "read_file" => {
                let Some(path_str) = args.get("path").and_then(|v| v.as_str()) else {
                    return "error: missing 'path' argument".to_string();
                };
                self.read_file_guarded(path_str).await
            }
            "git_diff" => {
                let base_diff_ref = format!("{}...HEAD", self.base_ref);
                let mut git_args: Vec<&str> = vec!["diff", &base_diff_ref];
                if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
                    git_args.push("--");
                    git_args.push(p);
                }
                run_git(self.dir.path(), &git_args).await
            }
            "git_log" => {
                let max_count = args
                    .get("max_count")
                    .and_then(|v: &serde_json::Value| v.as_u64())
                    .unwrap_or(20);
                let mc = max_count.to_string();
                let range = format!("{}..HEAD", self.base_ref);
                run_git(
                    self.dir.path(),
                    &[
                        "log",
                        &range,
                        "--format=commit %H%nAuthor: %an <%ae>%nDate: %ad%n%n%B%n---",
                        "--max-count",
                        &mc,
                    ],
                )
                .await
            }
            other => format!("unknown tool: {other}"),
        }
    }

    /// Read a file guarded by a path-traversal check.
    #[tracing::instrument(
        skip(self),
        fields(category = "ai_review_tool", tool = "read_file", path = %path_str)
    )]
    async fn read_file_guarded(&self, path_str: &str) -> String {
        let full_path = self.dir.path().join(path_str);
        // Canonicalize the target (this fails if the file doesn't exist,
        // which produces a usable error string).
        let Ok(canon) = tokio::task::spawn_blocking({
            let p = full_path.clone();
            move || p.canonicalize()
        })
        .await
        .unwrap_or(Err(std::io::Error::other("spawn_blocking failed")))
        else {
            return format!("error: cannot resolve path: {path_str}");
        };
        let Ok(root) = tokio::task::spawn_blocking({
            let p = self.dir.path().to_path_buf();
            move || p.canonicalize()
        })
        .await
        .unwrap_or(Err(std::io::Error::other("spawn_blocking failed")))
        else {
            return "error: internal workspace error".to_string();
        };
        if !canon.starts_with(&root) {
            return format!("error: path traversal rejected: {path_str}");
        }
        match tokio::fs::read_to_string(&canon).await {
            Ok(content) => truncate(&content),
            Err(e) => format!("error: cannot read {path_str}: {e}"),
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Run `git` in `dir` without exposing any GitHub token. Returns combined
/// stdout + stderr, truncated.
#[tracing::instrument(
    skip(dir, args),
    fields(category = "ai_review_tool", binary = "git")
)]
async fn run_git(dir: &Path, args: &[&str]) -> String {
    let mut cmd = TokioCommand::new("git");
    cmd.args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) => return format!("error: git command failed: {e}"),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() {
        stdout.into_owned()
    } else {
        format!("{stdout}\n{stderr}")
    };
    truncate(&combined)
}

/// Run `gh` with `GH_TOKEN` set, returning stdout on success.
#[tracing::instrument(
    skip(token, args),
    fields(category = "ai_review_tool", binary = "gh")
)]
async fn run_gh_stdout(
    dir: &Path,
    token: &str,
    args: &[&str],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = run_gh_raw(dir, Some(token), args).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh command failed: {stderr}").into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run `gh` with `GH_TOKEN`, returning `Err` on non-zero exit.
#[tracing::instrument(
    skip(token, args),
    fields(category = "ai_review_tool", binary = "gh")
)]
async fn run_gh_checked(
    dir: &Path,
    token: &str,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = run_gh_raw(dir, Some(token), args).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh command failed: {stderr}").into());
    }
    Ok(())
}

/// Run `git` with error-on-failure semantics (used during checkout).
#[tracing::instrument(
    skip(dir, args),
    fields(category = "ai_review_tool", binary = "git")
)]
async fn run_git_checked(
    dir: &Path,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = TokioCommand::new("git");
    cmd.args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git command failed: {stderr}").into());
    }
    Ok(())
}

/// Raw `gh` subprocess — the requester's token is set as `GH_TOKEN` only
/// when `token` is `Some`.
#[tracing::instrument(
    skip(token, args),
    fields(category = "ai_review_tool", binary = "gh")
)]
async fn run_gh_raw(
    dir: &Path,
    token: Option<&str>,
    args: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = TokioCommand::new("gh");
    cmd.args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(t) = token {
        cmd.env("GH_TOKEN", t);
    }
    Ok(cmd.output().await?)
}

// ── Output truncation ───────────────────────────────────────────────────────

pub(super) fn truncate(s: &str) -> String {
    if s.len() <= TOOL_OUTPUT_LIMIT {
        return s.to_string();
    }
    let mut end = TOOL_OUTPUT_LIMIT;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}[truncated]", &s[..end])
}
