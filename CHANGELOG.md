# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-06-07

### Added

- Grafana Tempo as the OTLP tracing backend, replacing Jaeger
- Grafana service in Docker Compose with the Tempo datasource auto-provisioned — no manual setup required
- `tempo.yaml` config with all storage paths explicitly set to `/var/tempo` — matching what the image pre-creates with the correct uid, works for both local and Docker use
- `broadcast_typing` OTel span so the Discord typing-indicator HTTP call shows up in traces instead of being swallowed by `handle_ai_channel_message` overhead

### Fixed

- Tokio Console: the gRPC server was never spawned — `.build()` returns `(ConsoleLayer, Server)` but only the layer was kept; the server was dropped, so there was nothing for `tokio-console` to connect to
- Docker: `COPY .env` baked secrets into the image layer; switched to `env_file` injection at runtime
- Grafana Tempo: all storage paths now explicitly configured to `/var/tempo` — the image pre-creates this directory for uid 10001; locally, create it once with `sudo mkdir -p /var/tempo && sudo chown $USER /var/tempo`
- Grafana Tempo 3.x: the new `live_store` module defaults to `/var/tempo` for its WAL and shutdown marker independently of `storage.trace`; now explicitly configured

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

[Unreleased]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/1git2clone/serenity-discord-bot/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1git2clone/serenity-discord-bot/releases/tag/v0.1.0
