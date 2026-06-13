# Architecture

A system overview for new contributors: the crate layout, how the bot starts,
how a message flows through the handlers, the two separate AI code paths, the
tool registry the review agent runs on, and the way Redis is treated as
optional throughout.

## Crate layout

- `src/` — the bot. One binary (`main.rs`), also exposed as `lib.rs` so the
  integration tests can link against it.
- `serenity_discord_bot_derive/` — a local proc-macro crate, pulled in as a
  path dependency. See [Derive macros](#derive-macros) below.

The stack is Rust 2024 edition (rustc 1.94), Serenity 0.12 and Poise 0.6 for
the Discord framework, PostgreSQL through sqlx, with optional Redis, AI, and
OpenTelemetry/Tokio-console layers gated behind Cargo features.

`prelude.rs` is a wide re-export hub: it aliases `poise::serenity_prelude` as
`serenity`, and re-exports sqlx, the bot-data statics, and the derive macros.
Most modules start with `use crate::prelude::*`.

## Startup sequence

`setup` in `main.rs` runs these steps in order. The order is deliberate —
later steps depend on earlier ones.

1. Register the command list globally.
2. Connect to PostgreSQL and wrap the pool in an `Arc`. Migrations run
   automatically at startup (sqlx), so a reachable database is brought
   up to date without a manual step.
3. (AI builds only) Build a `(name, description)` list from the commands that
   were actually registered, then call `ai::init_system_prompt(&commands)`.
   The system prompt's command list is derived from what is really registered,
   so it cannot go stale relative to the code.
4. (AI builds only) Force-evaluate the `AI_PROVIDER` and `AI_MAX_MSG_CONTEXT`
   statics. This happens after the prompt is set, so the command list is baked
   into the provider, and it happens now so that bad configuration fails at
   boot rather than on the first message.
5. (AI builds only) Initialize the registered auto-reply channels and the
   cache.
6. Spawn the reminder polling loop.
7. Return `Data { bot_user, bot_avatar, available_commands, pool }`. These are
   cached once at startup to avoid per-command HTTP and database lookups.
   `bot_avatar` rewrites the `.webp` face URL to `.png`.

### Fail-fast configuration

Required configuration is read through `LazyLock<String>` statics that panic
if the variable is missing (`AI_MODEL`, `AI_API_KEY`, `GITHUB_APP_ID`, and
friends, in `src/data/ai/config.rs` and `src/data/ai/review/config.rs`). They
are force-evaluated at startup so a misconfiguration crashes immediately with a
clear message instead of failing deep inside a request.

One sharp edge: a `LazyLock` that panics is poisoned for the rest of the
process — every later access re-panics. That is why `review_available`
(`ai_review.rs`) checks the GitHub variables with `std::env::var(...).is_err()`
before touching the panicking statics. Without that guard one missing variable
would permanently disable `/ai-review` for the whole run. Any new panicking
static used inside a command needs the same pre-check.

## Message data flow

A gateway message is handled in `event_handler`, which dispatches into the
helper functions:

```
gateway message
  └─ event_handler
       └─ helper_functions
            ├─ handle_database_message_processing  → XP award (random range,
            │                                         ~60s cooldown) + mention
            │                                         pattern reply embeds
            ├─ custom reaction matching (regex, redis-gated)
            └─ AI auto-reply (in registered channels, AI builds only)
```

XP is awarded per message inside `handle_database_message_processing`, and
mention-count patterns trigger reply embeds. The top-level `event_handler` is
deliberately not instrumented with a tracing span: it fires for every gateway
event, including presence updates, so a span there would be pure noise. Only
handled events carry spans.

Prefix commands are `hu` and `ht`, registered as case-insensitive regex
prefixes with mention-as-prefix enabled. Unrecognized prefix commands get
Levenshtein typo correction. Cooldowns are manual (`manual_cooldowns: true`);
the XP cooldown and the AI per-user rate limit are unrelated systems.

## The two AI code paths

There are two AI paths, at two different abstraction levels. They do not share
code and behave differently — do not conflate them.

### Path A — persona chat

`/ai`, the `/aichannel` auto-reply, and DMs. Goes through the `llm` crate via
the `AI_PROVIDER` static (`src/data/ai/provider.rs`). It is provider-agnostic:
it works with whichever `ai-<backend>` you compiled. It is plain chat
completion with a system persona and no tool calling.

### Path B — AI code review

`/ai-review`. This path bypasses the `llm` crate entirely and hand-rolls the
OpenAI/DeepSeek `/chat/completions` tool-calling protocol in
`src/data/ai/tools/client.rs`. The reason: as of `llm` 1.3.8 the DeepSeek
backend's `chat_with_tools` is `todo!()`, so there is nothing to call through
the crate.

The endpoint is hardcoded to `https://api.deepseek.com/chat/completions` and
auth uses `AI_API_KEY`. The consequence is that **`/ai-review` only works
against DeepSeek**, regardless of which `ai-<backend>` feature is enabled for
Path A. `AI_MODEL` must name a function-calling model (`deepseek-chat` works).
See [docs/ai.md](./ai.md) for the user-facing walkthrough and
[SECURITY.md](../SECURITY.md) for the threat model.

## The tool registry

`src/data/ai/tools/mod.rs` holds a small generic tool-calling abstraction used
by the review agent.

- `ToolSpec<Ctx>` is a wire definition (name, description, JSON-Schema
  parameters) plus a function-pointer handler. The handler is a plain `fn`
  pointer, not a `dyn Trait`, which sidesteps the boxing that an `async fn` in
  a trait would force. A tool is "data plus a function."
- `Ctx` is generic so handlers can borrow what they need. The review path's
  `ReviewCtx` carries the workspace, the PR coordinates, and the token.
- `ToolRegistry::dispatch` returns an error string for an unknown tool
  (`"unknown tool: <name>"`) rather than failing. This is the same convention
  the handlers use: tool errors are strings relayed back to the model so it can
  react, never an `Err` that aborts the agent loop. This is intentional and
  load-bearing for the loop's robustness.

## Redis-optional fallback model

`cache::conn()` returns `Option<ConnectionManager>`. `None` means `REDIS_URL`
is unset or the connection failed. The whole codebase treats Redis as
best-effort, and every Redis-backed feature has a single-instance fallback:

| Feature | With Redis | Without Redis (fallback) |
|---|---|---|
| AI context window | windowed list `ai:ctx:{channel}`, TTL 1800s | re-fetch recent messages from Discord every reply |
| Per-channel AI lock | `SET NX EX` lock, TTL 30s | no-op guard — cannot dedupe |
| AI user rate limit | `SET NX EX`, TTL 10s | never rate-limits |
| Global review guard | lock `ai:review_guard`, TTL 600s | no-op guard |

A single instance runs fine without Redis. Multi-instance deployments need it:
the locks and rate limits are coordination primitives that otherwise only hold
per-process.

`RedisLockGuard` is RAII — it releases its lock on `Drop` by spawning a task,
because `Drop` cannot be async. The release is best-effort and the TTL is the
real safety net. An empty-key guard is the no-op fallback and skips the spawn.

## Derive macros

`serenity_discord_bot_derive/` provides four small, concrete derives. This is
deduplication through shared helper functions over a spec, not a macro DSL.

- `IterateVariants` — adds `fn variants() -> &'static [Self]`.
- `DiscordEmoji` (attribute `emoji_id`) — `Display` renders `<:Name:id>`, plus
  `get_id()` and `get_variant_str()`.
- `Asset` (attributes `base_url`, `src_path`) — treats an enum like a
  filesystem; `Display` yields `base_url/src_path`, used for CDN asset URLs.
- `DatabaseEnum` — `Display` converts `PascalCase` variants to `snake_case`,
  plus an `.as_str()`.

## Observability

Tracing is layered. See [docs/observability.md](./observability.md) for the
full picture: the console layer's default filter, the separate OpenTelemetry
filter, the `category` span field, Tokio Console, and the Tempo/Grafana stack.
