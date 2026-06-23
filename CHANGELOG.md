# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `util-download` Cargo feature — `/util download <url> [start] [end]` downloads media via `yt-dlp`, optionally trims with `ffmpeg`, then 2-pass encodes targeting 8 MB for Discord attachment upload. Requires `yt-dlp` and `ffmpeg` (with `ffprobe`) on `PATH`
- `/util` parent command group — groups `/avatar`, `/uptime`, `/age`, and `/util download` (when `util-download` is enabled) under a single `/util` parent. The old standalone `/avatar`, `/uptime`, and `/age` commands remain registered for backward compatibility
- `ai-openrouter` Cargo feature — adds OpenRouter as a persona-chat backend alongside `ai-deepseek`, `ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, and `ai-groq`. Set `AI_API_KEY` to an OpenRouter key and `AI_MODEL` to an OpenRouter model id (e.g. `deepseek/deepseek-chat`)

### Removed

- `/ai-review` command and all supporting code — removed because automatic AI PR reviewers (this repo's own `/code-review ultra` / Codewhale) do a strictly better job. The command only worked against DeepSeek (bypassing the `llm` crate's `todo!()` tool-calling path), pulled in `jsonwebtoken` and `tempfile` deps, and required its own GitHub App, Postgres table, Redis guard, and device-flow OAuth dance — significant surface area for a redundant feature. The `ai_review_guilds` table is dropped via a new migration

### Fixed

- The `[replying to ...]` reply-context marker no longer leaks into the bot's visible replies, where it stacked into dozens of repeated copies before the actual message. The bot replies inline, so its own messages were rendered with the marker and stored as `assistant` turns; the model then learned its replies should start with `[replying to ...]` and parroted it. The marker is now kept off the bot's own turns (it's a cue for reading *other* people's reply links), and any marker the model still echoes is stripped from the response before it's sent
- Reply-context marker no longer nests/duplicates down the reply chain. `render_message` snippeted the parent via `render_message(parent)`, which re-prepended the parent's own `[replying to ...]` marker — so a reply-to-a-reply produced `[replying to A: [replying to B: ...]]` and walked the whole chain unbounded, contradicting the documented "one level deep". The parent is now snippeted from its *body* (`render_body`, content + embeds, no marker), so the marker is strictly one level deep

## [0.3.0] - 2026-06-14

### Added

- `/custom reaction add url:<url> pattern:<regex> anywhere:<bool>` and `/custom reaction add attachment:<file> pattern:<regex> anywhere:<bool>` — staff (Manage Channels) register an image + a Rust regex; any matching guild message gets a red embed reply showing the image. `anywhere: false` (the default) anchors the pattern to the full trimmed message; `anywhere: true` matches anywhere. Patterns are validated at register time: 512-char cap, compiled-automaton size limit, and empty-string-match rejection (all with human-readable error messages including a regex101 Rust-flavor link). Multiple reactions can fire on one message — all matches are sent, ordered by id. The confirmation reports the reaction's **per-guild number** (`#1`, `#2`, … within the server), not the global database id. Tenor/Giphy *page* links (`tenor.com/view/…`, `giphy.com/gifs/…`) are rejected at register time with a hint to paste the direct media URL instead — a page link is an HTML page, not an image, and renders as a blank box in an embed. Discord CDN attachment URLs have signing query params stripped at storage time; external URLs are stored as-is
- `/custom reaction list` — lists every reaction in the server (ephemeral) with its per-guild number, pattern, `anywhere` flag, and a short URL hint
- `/custom reaction remove name:<autocomplete>` — soft-deletes the reaction picked from an autocomplete dropdown backed by the Redis cache; the per-guild number is the parsed value and the pattern text is the visible label. Per-guild numbers are positional (1..N ordered by id), so removing one renumbers the rest — `list` always shows the current numbers
- Custom reactions are capped at 25 per guild (enforced at register time). All reads and writes route through a write-through Redis cache: a `cr:guilds` set provides an O(1) per-message short-circuit for guilds with no reactions; `cr:meta:{guild_id}` hashes hold the per-entry JSON; `cr:seeded` gates cold-start population from `fetch_all_live`. Degrades to direct DB queries when Redis is unavailable
- `/custom prompt add <text>` / `/custom prompt show` / `/custom prompt remove` — staff (Manage Channels) set a per-server instruction block appended to the AI's system prompt, without touching the shared Hu Tao persona. `show` is ephemeral; `remove` clears it. Persisted in the new `guild_ai_settings` table (one row per guild) and fronted by a best-effort Redis cache (`ai:guild_prompt:{guild_id}`, 30 min TTL) that negative-caches the no-prompt case so the per-message hot path stays off the DB; mutations invalidate the key. The prompt is injected as a leading turn in the guild's AI context; DMs have no guild and skip it
- Multi-instance support: when `TOTAL_SHARDS`, `SHARD_START`, and `SHARD_END` are set, the instance connects only its shard range so several instances can split the shard space; without them the bot starts as before. Over-provision the total (e.g. 16) and scale by redistributing ranges to avoid resharding
- Redis-backed coordination, replacing in-process state: the per-channel AI processing lock, the per-user AI rate limit, and the global one-review-at-a-time guard now use Redis (with token-checked release and TTL safety nets), so they hold across instances. Registered AI channels are cached in a Redis set seeded from Postgres. Everything degrades gracefully when `REDIS_URL` is unset — locks and rate limits become per-instance and lookups fall back to the database

### Changed

- The AI context now carries reply links: when a message is an inline reply, its rendered turn is prefixed with a `[replying to <author>: <snippet>]` marker (the parent flattened and truncated the same way as any other message), so the model can tell which earlier message an answer addresses instead of assuming the most recent line. One level deep; rides the existing content field with no Redis schema change
- Hu Tao's AI replies are now kept short — a sentence or two (often one line), the way Discord chatter actually reads — with room to go longer only when a question genuinely needs an explanation or steps
- `/ai-review` per-guild authorization is checked against Postgres directly instead of an in-memory set — a stale cached "enabled" was unsafe, and the command is rare enough that the query doesn't matter
- The AI tool-calling loop is now a reusable registry (`ToolSpec`/`ToolRegistry` in `data::ai::tools`) instead of being hand-wired inside `/ai-review`: each tool's model-facing definition and its handler live together, dispatch is by name instead of scattered `match` arms, and the DeepSeek tool-calling client moved into shared AI code so future tools (e.g. a chat-side search) can reuse the loop; behavior is unchanged

### Fixed

- Reminder delivery is now multi-instance safe: due reminders are claimed atomically (`FOR UPDATE SKIP LOCKED`), so concurrent instances can't DM the same reminder twice
- Hu Tao mention counting lost increments under concurrency — fetch-then-update replaced with a single atomic `UPDATE ... RETURNING`
- `SHARD_END` was documented as exclusive, but serenity's `start_shard_range` treats the range end as inclusive — adjacent instances both started the boundary shard (their sessions kept invalidating each other) and the last instance tried to start a nonexistent shard, looping on `Gateway(InvalidShardData)`. `SHARD_END` is now inclusive, and the range is validated at startup (`SHARD_START <= SHARD_END < TOTAL_SHARDS`) so a bad range fails fast instead of retrying forever

## [0.2.2] - 2026-06-12

### Added

- `/ai-review run url:<repo-url> pr:<n>` — requests an AI code review of a GitHub PR from Discord. A Hu Tao-themed review agent shallow-clones the PR, inspects it with read-only tools (`list_files`, `read_file`, `git_diff`, `git_log`, `pr_conversation`), and posts a structured review as a PR comment. The `pr_conversation` tool reads existing comments, reviews, and inline threads so the agent doesn't repeat prior feedback, and it reviews incrementally when it finds its own earlier `<!-- ai-review -->` comment — bugs are restless spirits, clean code crosses over peacefully. On first use (and after a restart or cache TTL) the requester links their GitHub account via the OAuth device flow (github.com/login/device); the bot verifies they have push access to the target repo before running. The review comment is posted by the bot's GitHub App, not the requester. User tokens are held in memory only — never written to disk. One review runs at a time. Requires a GitHub App: `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_APP_ID`, and `GITHUB_APP_PRIVATE_KEY_PATH` (see `.env.example`)
- `/ai-review enable` / `/ai-review disable` — per-server opt-in for the review command (Administrator only), persisted in the new `ai_review_guilds` table and cached in memory
- `/reminder delete` subcommand: cancels a pending reminder chosen from an autocomplete dropdown (filtered by message text); only the reminder's owner can delete it
- Reminders now store the resolved timezone at create time; `/reminder delete` autocomplete displays each reminder's fire time in that timezone instead of UTC

### Changed

- The 17 embed (GIF) commands now share one spec-driven implementation (`InteractionSpec`/`OnSelf` plus embed helpers) instead of one near-identical file each; behavior is unchanged

### Fixed

- Docker: the runtime image now includes `git` and the GitHub CLI — the default build features include `ai-deepseek`, so `/ai-review` would have failed inside the container with its subprocess dependencies missing

## [0.2.1] - 2026-06-08

### Added

- `/reminder` slash command group that DMs you at a set time: `create`, `list`, `search`, and `timezone`
- Per-user default timezone via `/reminder timezone`, saved per-server or globally; `create` resolves the zone from the explicit option, then the server default, then the global default, then UTC
- Timezone input accepts IANA names (`Europe/Sofia`), bare cities (`Sofia`), or GMT offsets (`GMT+2`, `+02:00`), with Discord autocomplete over the IANA database
- Reminder history: fired reminders are kept (capped at 100 per user) and browsable; `/reminder list` and `/reminder search` show them in an ephemeral, button-paginated embed (first/prev/next/last plus a jump-to-page modal)
- AI replies now also trigger when the bot is mentioned anywhere in a message, not only in DMs or `/aichannel` channels

### Fixed

- Docker: consolidated to a single `tempo.yaml` using `/var/tempo` paths — the image pre-creates this directory with the correct uid, so it works for both Docker and local use without a separate config file
- Docker: bumped the builder image to rust 1.94 to satisfy the sqlx 0.9 MSRV

## [0.2.0] - 2026-06-07

### Added

- Grafana Tempo as the OTLP tracing backend, replacing Jaeger
- Grafana service in Docker Compose with the Tempo datasource auto-provisioned — no manual setup required
- `tempo.yaml` config with local storage paths so Tempo can be run outside of compose without root
- `broadcast_typing` OTel span so the Discord typing-indicator HTTP call shows up in traces instead of being swallowed by `handle_ai_channel_message` overhead

### Fixed

- Tokio Console: the gRPC server was never spawned — `.build()` returns `(ConsoleLayer, Server)` but only the layer was kept; the server was dropped, so there was nothing for `tokio-console` to connect to
- Docker: `COPY .env` baked secrets into the image layer; switched to `env_file` injection at runtime
- Grafana Tempo: storage paths defaulted to `/var/tempo`, causing a permission error when running as a non-root user locally; switched to relative paths under `./tempo-data/`
- Grafana Tempo 3.x: the new `live_store` module also defaults to `/var/tempo` for its WAL and shutdown marker, independent of the `storage.trace` config; now explicitly configured

## [0.1.1] - 2025-12-01

### Added

- Migrated AI provider from Ollama to the [`llm` crate](https://crates.io/crates/llm), which supports DeepSeek, OpenAI, Anthropic, Google, Groq, and Ollama through a single compile-time feature flag
- `/aichannel` command to toggle a channel for automatic AI replies to every message
- Per-user AI rate limiting and per-channel processing lock to prevent reply storms
- Redis context caching for AI conversation windows; falls back to a Discord HTTP fetch on cold channels
- AI replies in DMs — no channel registration required
- Bot command list baked into the AI system prompt so the model can explain itself in character
- AI reads embed content (author, title, description, fields, footer, image presence) so command output embeds are visible to the model
- OTel spans for Redis calls, Discord HTTP, and LLM requests — latency is now attributable per layer

### Fixed

- Prefix commands no longer expose the `msg` rest argument as a slash command option

## [0.1.0] - 2024-02-25

### Added

- Initial release built on [Serenity](https://github.com/serenity-rs/serenity) and [Poise](https://github.com/serenity-rs/poise)
- Slash and prefix commands (`hu`, `ht`, bot mention as prefix), case-insensitive
- Levenshtein distance typo correction for unrecognised prefix commands
- Embed interaction commands: pat, hug, kiss, slap, punch, bonk, nom, kill, kick, bury, selfbury, peek, avatar, drive, chair, boom, quote, cookie
- XP levelling system backed by PostgreSQL with a 60-second per-user cooldown (moka TTL cache)
- `/level` — current XP, level, and progress for a user
- `/toplevels` — server leaderboard embed (~1.8s, down from ~3.5s via `try_join_all`)
- `/help` — lists all registered commands
- `/uptime` — bot uptime embed
- Discord emoji handling via a custom `discord_emoji` proc-macro derive
- Bot mention counting stored in the database
- OpenTelemetry tracing as an optional `opentelemetry` feature
- Tokio Console as an optional `tokio_console` feature
- Docker Compose setup with PostgreSQL and Jaeger
- SQLite → PostgreSQL migration

[Unreleased]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1git2clone/serenity-discord-bot/releases/tag/v0.1.0
