# Serenity Discord Bot

[![GH_Build Icon]][GH_Build Status]&emsp;[![Build Icon]][Build Status]&emsp;[![License Icon]][LICENSE]

[GH_Build Icon]: https://img.shields.io/github/actions/workflow/status/1git2clone/serenity-discord-bot/rust-and-docker.yml?branch=main
[GH_Build Status]: https://github.com/1git2clone/serenity-discord-bot/actions?query=branch%3Amain
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

## Features

- `/help` — lists every registered command
- Embed interaction commands: pat, hug, kiss, slap, punch, bonk, nom, kill, kick, bury, peek, avatar, drive, chair, boom, quote
- XP levelling with a 60-second cooldown, stored in PostgreSQL
- `/toplevels` — server leaderboard embed
- `/uptime` — bot uptime
- `/reminder` — schedule a DM for later (`create`/`list`/`search`), with a saveable default timezone (`/reminder timezone`, per-server or global) and browsable, paginated history
- Levenshtein-distance typo correction on unrecognised prefix commands

### Optional features

#### AI

An in-character Hu Tao persona powered by the [llm crate](https://crates.io/crates/llm), which supports every mainstream provider. The backend is chosen at compile time — enable exactly one of: `ai-deepseek`, `ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, `ai-groq`.

Set `AI_MODEL` and `AI_API_KEY` (hosted backends) in `.env` — see [`.env.example`](./.env.example) for all variables.

- `/ai` — one-off prompt in any channel or DM
- `/aichannel` — toggle a channel where the bot auto-replies to every message (requires Manage Channels)
- Set `REDIS_URL` to keep conversation context in Redis; without it the bot re-fetches recent messages from Discord on every reply

![AI channel demo](./assets/ai-channel-demo.png)

![AI DM demo](./assets/ai-dm-demo.png)

#### Tokio Console

Task-level async runtime inspection via [Tokio Console](https://github.com/tokio-rs/console):

```sh
RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio_console
```

![tokio-console task view](./assets/tokio-console-demo.png)

#### Telemetry

Distributed tracing via OpenTelemetry — backend-agnostic, so you can point it at any OTLP-compatible collector. The compose setup ships with [Grafana Tempo](https://grafana.com/oss/tempo/) and Grafana pre-wired as the UI. To run Tempo manually (create `/var/tempo` once with your user as owner):

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
2. Run:

```sh
cargo run --release
# or, to enable specific features:
cargo run --release --features='<your-features>'
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
