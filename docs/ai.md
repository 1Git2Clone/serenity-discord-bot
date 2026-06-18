# AI features

An operator and user guide for the optional AI features: the in-character Hu
Tao persona chat, auto-reply channels, the conversation context window, and the
GitHub PR code review agent.

There are two distinct AI paths with different behavior. The persona chat is
provider-agnostic; the code review agent only works against DeepSeek. The
[architecture overview](./architecture.md#the-two-ai-code-paths) explains why.

## Enabling AI at build time

AI is compiled in via Cargo features. `ai` is a meta-feature: it also enables
`redis`, and on its own it does not pick a provider. You must enable exactly
one backend:

```
ai-deepseek  ai-ollama  ai-anthropic  ai-openai  ai-google  ai-groq
```

Building with `--features ai` and no backend fails with a `compile_error!`.

```sh
cargo run --release --features ai-deepseek
```

Set `AI_MODEL` (required when AI is built) and, for hosted backends,
`AI_API_KEY`. `ai-ollama` does not need a key and can point at a custom base
URL with `AI_CHAT_ENDPOINT`. See [configuration](./configuration.md) for the
full variable list.

## Persona chat

Provider-agnostic, plain chat completion through the `llm` crate. No tool
calling.

- `/ai prompt:<text>` — a one-off prompt in any channel or DM.
- `/aichannel` — toggle a channel where the bot auto-replies to every message.
  Requires Manage Channels. Registered channels survive restarts.

Persona chat also runs in DMs.

### Per-guild custom prompt

`/custom prompt` appends a server-specific instruction block to the shared Hu
Tao system prompt without replacing the persona. Requires Manage Channels.

- `/custom prompt add <text>` — set or replace the guild's extra prompt.
- `/custom prompt show` — show the current one (ephemeral).
- `/custom prompt remove` — clear it.

The extra prompt is injected as a leading turn ahead of the conversation, so it
applies to both `/ai` and `/aichannel` auto-replies. It is stored in
`guild_ai_settings` and cached in Redis (`ai:guild_prompt:{guild}`, TTL 1800s,
DB authoritative); the no-prompt case is negative-cached so the per-message path
stays off the DB. DMs have no guild, so they use the base persona only.

### Context window

Conversation context is kept per channel so the persona can follow a thread.
The behavior depends on whether Redis is configured.

- With `REDIS_URL` set, recent turns are stored as a windowed Redis list
  (`ai:ctx:{channel}`, TTL 1800s), sized by `AI_MAX_MSG_CONTEXT` (default 10,
  cap 100).
- Without Redis, the bot re-fetches recent messages from Discord on every
  reply instead.

Three details worth knowing:

- **Warm channels only.** `record_message` appends a message to the window only
  if the window already exists. Channels become "warm" after a prior AI
  interaction; a cold channel is seeded on demand by a one-off Discord fetch
  the first time the bot needs context there.
- **Embeds are flattened.** Most command outputs are embed-only with empty
  message `content`. The context layer renders embeds to text and appends them,
  so the model sees command output instead of nothing.
- **Replies carry their parent.** When a message is an inline reply, its
  rendered turn is prefixed with `[replying to <author>: <snippet>]`, where the
  snippet is the parent's *body* (content + embeds) flattened and truncated the
  same way — but **not** the parent's own reply marker. This is what keeps it
  strictly one level deep: snippeting the parent's fully-rendered turn would
  re-include the parent's marker and nest unbounded down the reply chain. A
  deleted or unresolved parent is skipped.

Speakers are distinguished in the window: the bot's own messages become
`assistant` turns, and everyone else becomes a `user` turn prefixed with their
display name.

### Rate limits and locks

- Per-user AI rate limit: 10s (`ai:rl:{user}`), Redis-backed. Without Redis the
  bot never rate-limits.
- Per-channel AI lock: 30s (`ai:ch_lock:{ch}`), to dedupe concurrent replies in
  one channel. Without Redis this is a no-op.

These are separate from the XP levelling cooldown (~60s), which is an unrelated
system.

## AI code review

`/ai-review` runs a Hu Tao-themed review agent against a GitHub pull request and
posts the review as a PR comment.

> **Limitation.** `/ai-review` only works against DeepSeek. It bypasses the
> `llm` crate and talks to `https://api.deepseek.com/chat/completions` directly,
> regardless of which `ai-<backend>` you compiled for persona chat. `AI_MODEL`
> must name a function-calling model — `deepseek-chat` works. The host also
> needs `git` and the [GitHub CLI](https://cli.github.com/) (`gh`) installed;
> the agent shells out to them.

### Commands

- `/ai-review run url:<repo-url> pr:<n>` — review a PR. Reviews run one at a
  time across the deployment.
- `/ai-review enable` / `/ai-review disable` — admin-only, per-server opt-in.
  The setting is stored in PostgreSQL and survives restarts.

Until the setup below is complete, `run` reports that the feature is not
configured.

### Setup

On top of the AI feature:

1. Install `git` and `gh` on the host. No `gh auth login` is needed — the agent
   manages its own credentials.
2. Create a GitHub App (Settings → Developer settings → GitHub Apps) with
   Device Flow enabled and Pull requests: read & write permission. Install it
   on the repositories you want reviewable.
3. Set in `.env`:
   - `GITHUB_OAUTH_CLIENT_ID` — the app client ID (`Iv` prefix).
   - `GITHUB_APP_ID` — the numeric app ID.
   - `GITHUB_APP_PRIVATE_KEY_PATH` — path to a generated `.pem`.
   - `GITHUB_OAUTH_SCOPE` — set to `repo` for private-repo reviews; the default
     `public_repo` is enough for public repos.
4. Optionally tune `AI_REVIEW_MAX_ITERATIONS` (default 20),
   `AI_REVIEW_TIMEOUT_SECS` (default 600), and `GITHUB_TOKEN_TTL_SECS`
   (default 3600).
5. An administrator runs `/ai-review enable` in the server.

### First use (device flow)

The first time a user runs `/ai-review run` — and again after the in-memory
token TTL expires or the bot restarts — they receive an ephemeral message with
a `github.com/login/device` link and a short code. After they approve, the bot
verifies they have push permission on the target repo before starting the
review.

The PR comment is posted by the bot's GitHub App, not by the requester. The
user token is used only for the permission check, is never written to disk or a
database, and expires after the TTL. See [SECURITY.md](../SECURITY.md) for the
two-token model and the sandbox guarantees.

Reviews are advisory. The agent is told it may be wrong, and the review text
says so — verify its claims against the code.
