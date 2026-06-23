# Serenity Discord Bot

[![GH_Build Icon]][GH_Build Status]&emsp;[![Build Icon]][Build Status]&emsp;[![Coverage Icon]][Coverage Status]&emsp;[![License Icon]][LICENSE]

[GH_Build Icon]: https://img.shields.io/github/actions/workflow/status/1git2clone/serenity-discord-bot/full.yml?branch=main
[GH_Build Status]: https://github.com/1git2clone/serenity-discord-bot/actions?query=branch%3Amain
[Coverage Icon]: https://codecov.io/gh/1git2clone/serenity-discord-bot/branch/main/graph/badge.svg
[Coverage Status]: https://codecov.io/gh/1git2clone/serenity-discord-bot
[Build Icon]: https://gitlab.com/1k2s/serenity-discord-bot/badges/main/pipeline.svg
[Build Status]: https://gitlab.com/1k2s/serenity-discord-bot/-/pipelines
[License Icon]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[License]: LICENSE

<!-- markdownlint-disable MD033 -->
<p>
  <img
    height="50px"
    src="https://codeberg.org/1Kill2Steal/skill-icons/raw/branch/main/icons/Rust.svg"
    alt="Rust"
  />
  <img
    height="50px"
    src="https://codeberg.org/1Kill2Steal/skill-icons/raw/branch/main/icons/PostgreSQL-Dark.svg"
    alt="PostgreSQL"
  />
  <img
    height="50px"
    src="https://codeberg.org/1Kill2Steal/skill-icons/raw/branch/main/icons/Docker.svg"
    alt="Docker"
  />
  <img
    height="50px"
    src="https://codeberg.org/1Kill2Steal/skill-icons/raw/branch/main/icons/Redis-Dark.svg"
    alt="Redis"
  />
  <img
    height="50px"
    src="https://codeberg.org/1Kill2Steal/skill-icons/raw/branch/main/icons/Grafana-Dark.svg"
    alt="Grafana"
  />
</p>
<!-- markdownlint-enable MD033 -->

![Bot profile](./assets/bot-profile.png)

[Try the bot out!](https://discord.com/oauth2/authorize?client_id=1211325231659089920 "Bot ID: 1211325231659089920")

A Hu Tao-themed Discord bot built with [Serenity](https://github.com/serenity-rs/serenity) and [Poise](https://github.com/serenity-rs/poise). Responds to both slash and prefix commands (`hu`, `ht`), with persistent XP levelling backed by PostgreSQL and an optional AI persona that stays in character across a full channel conversation window.

The minimal build needs only a Discord token and PostgreSQL; AI, Redis, and
telemetry are opt-in Cargo features. Run it with:

```sh
cargo run --release
```

For deeper detail, see the [documentation](#documentation): architecture,
configuration, deployment, AI features, and observability.

## Features

- `/help` — lists every registered command
- Embed interaction commands: tieup, pat, hug, kiss, slap, punch, bonk, nom, kill, kick, bury, selfbury, peek, avatar, drive, chair, boom, quote
- XP levelling with a 60-second cooldown, stored in PostgreSQL — `/level` for a member's level, `/toplevels` for the server leaderboard
- `/reminder` — schedule a DM for later (`create`/`list`/`search`/`delete`), with a saveable default timezone (`/reminder timezone`, per-server or global) and browsable, paginated history
- `/age` — your or another member's account creation date
- `/cookie` — give someone a cookie
- `/uptime` — bot uptime
- `/custom reaction add url:<url> pattern:<regex> [anywhere:<bool>]` — register an image + Rust regex; whenever a guild message matches, the bot replies with the image in an embed. `anywhere: false` (default) anchors to the full message; `anywhere: true` matches anywhere. The confirmation shows the reaction's per-guild number (`#1`, `#2`, … within the server). Tenor/Giphy *page* links (`tenor.com/view/…`, `giphy.com/gifs/…`) are rejected — paste the direct media URL (`media.tenor.com/....gif`) instead. Requires Manage Channels. Per-guild cap: 25
- `/custom reaction list` — list the server's reactions (ephemeral) with their per-guild numbers, patterns, and `anywhere` flags (Manage Channels)
- `/custom reaction remove name:<autocomplete>` — soft-delete the reaction picked by its per-guild number from autocomplete (Manage Channels, cache-backed). Numbers are positional, so removing one renumbers the rest
- Levenshtein-distance typo correction on unrecognised prefix commands

### Optional features

Everything optional is a Cargo feature. `ai` is a meta-feature — enabling it
also enables `redis` — and it requires picking exactly one `ai-<backend>`.

| Feature | What it adds | Notes |
|---|---|---|
| (core) | Commands, XP, reminders, custom reactions | Needs only `BOT_TOKEN` + PostgreSQL. |
| `redis` | Cross-instance AI context, locks, rate limits | Standalone. Single instance works without it. |
| `ai` | The AI persona and `/ai-review` | Meta-feature: also enables `redis`. Needs a backend (below). |
| `ai-<backend>` | The AI provider | Exactly one of `ai-deepseek`, `ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, `ai-groq`, `ai-openrouter`. Mandatory when `ai` is on. |
| `opentelemetry` | OTLP trace export | Point at any OTLP collector; compose ships Tempo + Grafana. |
| `tokio_console` | Tokio Console runtime inspection | Needs `RUSTFLAGS="--cfg tokio_unstable"`. |

Building with `--features ai` and no backend stops at a `compile_error!` by
design. See [docs/configuration.md](./docs/configuration.md) for the
environment variables each feature reads.

#### AI

An in-character Hu Tao persona powered by the [llm crate](https://crates.io/crates/llm), which supports every mainstream provider. The backend is chosen at compile time — enable exactly one of: `ai-deepseek`, `ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, `ai-groq`, `ai-openrouter`.

Set `AI_MODEL` and `AI_API_KEY` (hosted backends) in `.env` — see [`.env.example`](./.env.example) for all variables.

- `/ai` — one-off prompt in any channel or DM
- `/aichannel` — toggle a channel where the bot auto-replies to every message (requires Manage Channels)
- `/custom prompt add|show|remove` — set a per-server instruction block appended to the AI's system prompt, applied to both `/ai` and auto-replies (requires Manage Channels)
- `/ai-review` — AI code review of a GitHub PR (see below)
- Set `REDIS_URL` to keep conversation context in Redis and to share AI locks and rate limits across bot instances; without it the bot re-fetches recent messages from Discord on every reply and coordination is per-instance

![AI channel demo](./assets/ai-channel-demo.png)

![AI DM demo](./assets/ai-dm-demo.png)

#### AI code review

`/ai-review run url:<repo-url> pr:<n>` — a Hu Tao-themed agent shallow-clones a
GitHub PR, inspects it with read-only tools (`list_files`, `read_file`,
`git_diff`, `git_log`, `pr_conversation`), and posts a structured review as a PR
comment. It needs a GitHub App, per-server admin opt-in (`/ai-review enable`),
and device-flow authorization on first use. Reviews run one at a time and are
advisory.

Limitations: `/ai-review` only works against DeepSeek (it talks to the DeepSeek
endpoint directly, regardless of which `ai-<backend>` you built), `AI_MODEL`
must be a function-calling model (`deepseek-chat`), and the host needs `git` and
the [GitHub CLI](https://cli.github.com/) (`gh`) installed.

See [docs/ai.md](./docs/ai.md) for the full setup and usage walkthrough, and
[SECURITY.md](./SECURITY.md) for the two-token model and sandbox guarantees.

#### Tokio Console

Task-level async runtime inspection via [Tokio Console](https://github.com/tokio-rs/console):

```sh
RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio_console
```

![tokio-console task view](./assets/tokio-console-demo.png)

#### Telemetry

Distributed tracing via OpenTelemetry — backend-agnostic, so you can point it at any OTLP-compatible collector. The compose setup ships with [Grafana Tempo](https://grafana.com/oss/tempo/) and Grafana pre-wired as the UI. See [docs/observability.md](./docs/observability.md) for the tracing layers and the `category` span field. To run Tempo manually (create `/var/tempo` once with your user as owner):

```sh
sudo mkdir -p /var/tempo && sudo chown $USER /var/tempo
tempo -config.file=./tempo.yaml
```

![otel-tui trace view](./assets/otel-tui-trace.png)

![Grafana Tempo trace view](./assets/grafana-tempo-trace.png)

![Grafana Tempo flame graph](./assets/grafana-tempo-flamegraph.png)

![Grafana Tempo span details](./assets/grafana-tempo-span-details.png)

## Setting up

1. Copy `.env.example` to `.env` and fill in the values.
2. Have PostgreSQL running and reachable at `DATABASE_URL` — migrations run
   automatically at startup.
3. Run:

```sh
cargo run --release
# or, to enable specific features:
cargo run --release --features='<your-features>'
```

To run the telemetry stack (Grafana Tempo + Grafana) in containers while
running the bot natively:

```sh
docker-compose -f docker-compose.infra.yml up -d
```

### Docker Compose

The compose file brings up PostgreSQL, Redis, Grafana Tempo, and Grafana alongside the bot:

```sh
docker-compose up -d
```

> [!IMPORTANT]
> Make sure you aren't running PostgreSQL or Grafana Tempo locally due to port
> conflicts!

> [!NOTE]
> The [`Dockerfile`](./Dockerfile) builds with the features listed in its `FEATURES`
> arg (defaults to `ai-deepseek opentelemetry tokio_console`). Override via the
> compose build args to change provider or feature set.

## Documentation

- [docs/architecture.md](./docs/architecture.md) — crate layout, startup,
  message flow, the two AI paths, and the Redis-optional model.
- [docs/configuration.md](./docs/configuration.md) — full environment-variable
  reference.
- [docs/deployment.md](./docs/deployment.md) — native, Docker Compose,
  sharding, and the production topology.
- [docs/ai.md](./docs/ai.md) — enabling AI, the context window, and the
  `/ai-review` walkthrough.
- [docs/observability.md](./docs/observability.md) — tracing, Tokio Console,
  and OpenTelemetry.

See [SECURITY.md](./SECURITY.md) for the secrets inventory and AI review threat
model, and [CONTRIBUTORS.md](./CONTRIBUTORS.md) to set up a dev environment.
