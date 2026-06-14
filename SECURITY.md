# Security Policy

## Reporting a vulnerability

Report suspected vulnerabilities privately rather than opening a public issue.
Use GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
on this repository (Security → Report a vulnerability), or contact the
maintainer directly.

<!-- Replace with a real private contact if one is established. -->

Please include the affected version or commit, reproduction steps, and the
impact you observed. We aim to acknowledge reports and will coordinate a fix
and disclosure timeline with you.

## Secrets inventory

The bot handles four sensitive credentials. None are ever logged.

| Secret | Where it lives | Notes |
|---|---|---|
| `BOT_TOKEN` | `.env` / environment | Discord bot token. |
| `AI_API_KEY` | `.env` / environment | AI provider key (hosted backends). |
| GitHub App private key | a `.pem` file at `GITHUB_APP_PRIVATE_KEY_PATH` | The bot's crown jewel — see below. |
| GitHub user tokens | in-memory only | Device-flow tokens; never persisted (see threat model). |

`.env` and `*.pem` are gitignored. The GitHub App private key is the most
sensitive item: anyone holding it can act as the App on every installation.
Keep it off version control and off shared storage.

## AI code review threat model

`/ai-review` clones an arbitrary GitHub PR and feeds its contents to a language
model. The design assumes the PR may be hostile. The protections below are the
reason that is safe.

### Two-token separation

The command uses two different GitHub tokens for two different purposes. This
separation is the core of the design.

1. **User token** — obtained through GitHub *device flow* OAuth. It is used for
   one thing only: verifying that the requester has push, maintain, or admin
   permission on the target repo. It is **never written to disk or a database**
   — it is held in an in-memory cache keyed by Discord user ID, with a TTL of
   `GITHUB_TOKEN_TTL_SECS` (default 3600s). After the TTL or a restart, the user
   re-authorizes.
2. **GitHub App installation token** — the bot signs a short-lived RS256 JWT
   (exp 600s) from its app private key, discovers the installation ID
   dynamically, and exchanges the JWT for an installation token. **The PR
   comment is posted by the App, not by the requester.**

So the user token authorizes the *person*; the App token performs the *action*.

### Access gates

- **Per-server opt-in**, stored in PostgreSQL (`ai_review_guilds`). Only an
  admin can `/ai-review enable` or `disable`, and the setting survives
  restarts. The `run` command is gated by a `review_available` check.
- **Global single-flight guard** (Redis lock, TTL 600s): one review runs at a
  time across the deployment.
- **OAuth scope** defaults to `public_repo`; set `GITHUB_OAUTH_SCOPE=repo` only
  when private-repo reviews are needed.
- The review config variables are all optional and only needed to *use*
  `/ai-review`. If any are missing, `review_available` reports the feature as
  not configured rather than crashing.

### Sandbox guarantees

The review runs against untrusted PR contents in a sandbox:

- **Tempdir workspace.** Work happens in a `tempfile::TempDir`, a shallow clone
  (`--depth=50`), auto-deleted on drop.
- **Read-only tools only.** The model is given `list_files`, `read_file`,
  `git_diff`, `git_log`, and `pr_conversation`. There is no write or shell tool.
- **Path-traversal defense.** `read_file` canonicalizes the requested path and
  the workspace root and rejects anything that does not stay within the root.
  Tested against `../../../etc/passwd`.
- **Argument-injection defense.** Owner and repo names are restricted to
  GitHub's name charset and rejected if they start with `-`, because they become
  `git`/`gh` subprocess arguments and must not be parseable as flags.
- **Token never in argv.** The `gh` token is passed via the `GH_TOKEN`
  environment variable, never on the command line, so it cannot leak through
  process listings. `run_git` is never given the token at all.
- **Output truncation.** Every tool result is capped at 64 KiB, truncated on a
  UTF-8 character boundary.
- **Bounded loop.** The agent loop is bounded by both
  `AI_REVIEW_MAX_ITERATIONS` (default 20 tool rounds) and a wall-clock
  `AI_REVIEW_TIMEOUT_SECS` (default 600s).

The diff handed to the model comes from the GitHub API, not the local
`git_diff` tool: a shallow clone may not contain the merge base, so a local diff
could be silently wrong.

### Prompt-injection posture

Defense here is partly persona and partly structure, and is explicitly
best-effort:

- The review system prompt casts Hu Tao as a character who has never seen a
  computer and is told to ignore any instructions embedded in the diff or
  files.
- The same reminder is repeated in the user message, because models tend to
  obey the latest user turn more than the system prompt.
- Reviews are advisory. The output states it may be wrong and asks the reader to
  verify. Treat a review as a suggestion, not a gate.

## Operational guidance

- **Rotate the GitHub App private key** if it is ever exposed. It grants
  App-level access to every installation.
- **Least privilege.** Leave `GITHUB_OAUTH_SCOPE` at `public_repo` unless
  private-repo reviews are required; only then set `repo`.
- **Keep `git` and `gh` patched** on the host, since the review agent shells out
  to them.
