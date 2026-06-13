use std::{path::Path, process::Stdio};

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
        run_git_checked(dir.path(), &["fetch", "origin", &format!("pull/{pr}/head")]).await?;
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
    //
    // One method per tool, each taking the model's JSON arguments and returning
    // a textual result. Errors are returned as strings (not `Result`) so the
    // agent loop can relay them to the model. The tool registry in `agent.rs`
    // maps tool names to these methods.

    /// `list_files`: git-tracked files under `path` (default repo root).
    #[tracing::instrument(
        skip(self, args),
        fields(category = "ai_review_tool", tool = "list_files")
    )]
    pub async fn list_files(&self, args: Value) -> String {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        run_git(self.dir.path(), &["ls-files", path]).await
    }

    /// `read_file`: file contents, guarded against path traversal.
    #[tracing::instrument(
        skip(self, args),
        fields(category = "ai_review_tool", tool = "read_file")
    )]
    pub async fn read_file(&self, args: Value) -> String {
        let Some(path_str) = args.get("path").and_then(|v| v.as_str()) else {
            return "error: missing 'path' argument".to_string();
        };
        self.read_file_guarded(path_str).await
    }

    /// `git_diff`: base...HEAD diff, optionally scoped to `path`.
    #[tracing::instrument(
        skip(self, args),
        fields(category = "ai_review_tool", tool = "git_diff")
    )]
    pub async fn git_diff(&self, args: Value) -> String {
        let base_diff_ref = format!("{}...HEAD", self.base_ref);
        let mut git_args: Vec<&str> = vec!["diff", &base_diff_ref];
        if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
            git_args.push("--");
            git_args.push(p);
        }
        run_git(self.dir.path(), &git_args).await
    }

    /// `git_log`: up to `max_count` (default 20) commits on the PR but not base.
    #[tracing::instrument(
        skip(self, args),
        fields(category = "ai_review_tool", tool = "git_log")
    )]
    pub async fn git_log(&self, args: Value) -> String {
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
        .unwrap_or(Err(std::io::Error::other("spawn_blocking failed"))) else {
            return format!("error: cannot resolve path: {path_str}");
        };
        let Ok(root) = tokio::task::spawn_blocking({
            let p = self.dir.path().to_path_buf();
            move || p.canonicalize()
        })
        .await
        .unwrap_or(Err(std::io::Error::other("spawn_blocking failed"))) else {
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
#[tracing::instrument(skip(dir, args), fields(category = "ai_review_tool", binary = "git"))]
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
#[tracing::instrument(skip(token, args), fields(category = "ai_review_tool", binary = "gh"))]
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
#[tracing::instrument(skip(token, args), fields(category = "ai_review_tool", binary = "gh"))]
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
#[tracing::instrument(skip(dir, args), fields(category = "ai_review_tool", binary = "git"))]
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
#[tracing::instrument(skip(token, args), fields(category = "ai_review_tool", binary = "gh"))]
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

// ── Test helpers ────────────────────────────────────────────────────────────

/// A local two-commit git repo standing in for a PR checkout: the `base`
/// branch holds the first commit, HEAD has one commit on top of it. Shared
/// between this module's tests and the agent-loop tests in `agent.rs`.
#[cfg(test)]
pub(super) async fn test_workspace() -> Result<Workspace, Box<dyn std::error::Error + Send + Sync>>
{
    let dir = tempfile::tempdir()?;
    let p = dir.path();

    run_git_checked(p, &["init", "-b", "main"]).await?;
    run_git_checked(p, &["config", "user.email", "test@test"]).await?;
    run_git_checked(p, &["config", "user.name", "test"]).await?;

    tokio::fs::write(p.join("a.txt"), "base content\n").await?;
    run_git_checked(p, &["add", "."]).await?;
    run_git_checked(p, &["commit", "-m", "base commit"]).await?;
    run_git_checked(p, &["branch", "base"]).await?;

    tokio::fs::write(p.join("a.txt"), "changed content\n").await?;
    run_git_checked(p, &["commit", "-am", "pr commit"]).await?;

    Ok(Workspace {
        dir,
        base_ref: "base".to_string(),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[tokio::test]
    async fn list_files_lists_tracked_files() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.list_files(serde_json::json!({})).await;
        assert!(out.contains("a.txt"));
        Ok(())
    }

    #[tokio::test]
    async fn read_file_returns_contents() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.read_file(serde_json::json!({"path": "a.txt"})).await;
        assert_eq!(out, "changed content\n");
        Ok(())
    }

    #[tokio::test]
    async fn read_file_missing_path_arg() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.read_file(serde_json::json!({})).await;
        assert!(out.starts_with("error: missing 'path'"));
        Ok(())
    }

    #[tokio::test]
    async fn read_file_nonexistent() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.read_file(serde_json::json!({"path": "nope.txt"})).await;
        assert!(out.starts_with("error: cannot resolve path"));
        Ok(())
    }

    #[tokio::test]
    async fn read_file_rejects_path_traversal() -> TestResult {
        let ws = test_workspace().await?;
        // /etc/passwd exists and canonicalizes outside the workspace root.
        let out = ws
            .read_file(serde_json::json!({"path": "../../../../../../etc/passwd"}))
            .await;
        assert!(out.starts_with("error: path traversal rejected"));
        Ok(())
    }

    #[tokio::test]
    async fn git_diff_against_base() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.git_diff(serde_json::json!({})).await;
        assert!(out.contains("-base content"));
        assert!(out.contains("+changed content"));

        let scoped = ws.git_diff(serde_json::json!({"path": "a.txt"})).await;
        assert!(scoped.contains("+changed content"));
        Ok(())
    }

    #[tokio::test]
    async fn git_log_shows_pr_commits_only() -> TestResult {
        let ws = test_workspace().await?;
        let out = ws.git_log(serde_json::json!({})).await;
        assert!(out.contains("pr commit"));
        assert!(!out.contains("base commit"));
        Ok(())
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello"), "hello");
    }

    #[test]
    fn truncate_long_string_appends_marker() {
        let long = "a".repeat(TOOL_OUTPUT_LIMIT + 10);
        let out = truncate(&long);
        assert!(out.ends_with("[truncated]"));
        assert_eq!(out.len(), TOOL_OUTPUT_LIMIT + "[truncated]".len());
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        // 'é' is 2 bytes; an odd limit can't fall mid-character.
        let long = "é".repeat(TOOL_OUTPUT_LIMIT);
        let out = truncate(&long);
        assert!(out.ends_with("[truncated]"));
    }
}
