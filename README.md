# Serenity Discord Bot

[![GH_Build Icon]][GH_Build Status]&emsp;[![Build Icon]][Build Status]&emsp;[![License Icon]][LICENSE]

[GH_Build Icon]: https://img.shields.io/github/actions/workflow/status/1git2clone/serenity-discord-bot/rust.yml?branch=main
[GH_Build Status]: https://github.com/1git2clone/serenity-discord-bot/actions?query=branch%3Amaster
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
</p>
<!-- markdownlint-enable MD033 -->

## Features overview

This is a list of some of the Bots available features. For a more comprehensive
list you can always check out the code with all the commands or use the `help`
command.

- A help command containing all the bot commands.
- A bunch of embed interaction commands (like pats, hugs and etc.).
- A levelling system using a PostgreSQL connection which works with an XP
  cooldown (default is 60 seconds).
- The leveling system has a nice topranks command which gives a cool-looking embed!
- A bot uptime command.
- Additional [optional features](#optional-features).

### Optional features

#### AI

There's an optional `ai` feature (which can be enabled with `--features="ai"`)
using [Ollama](https://docs.ollama.com/quickstart).

To use it you simply need to run:

```sh
ollama pull gwen2.5:1.5b # <- You can use any model.
ollama serve
```

<!-- markdownlint-disable MD028 - False positive -->

> [!NOTE]
> You can use any model you like, just make sure to set it in the
> [`src/data/ai.rs`](./src/data/ai.rs) at
> `crate::data::ai::OllamaRequest::DEFAULT_MODEL`.

> [!NOTE]
> If you also wish to deploy Ollama on a Docker container for example and want
> to change the POST request URL, feel free to edit
> `crate::data::ai::OllamaRequest::CHAT_ENDPOINT` at
> [`src/data/ai.rs`](./src/data/ai.rs).

<!-- markdownlint-enable MD028 -->

#### Tokio Console

You can also enable the [Tokio Console](https://github.com/tokio-rs/console)
feature by compiling the bot with `--features="tokio_console"`.

> [!NOTE]
> Make sure to also compile with `RUSTFLAGS="--cfg tokio_unstable"` if you
> choose to do so.

#### Telemetry

The project uses back-end agnostic OpenTelemetry meaning you can choose your
preferred back-end if you choose to turn the `opentelemetry` feature flag on.

## Setting up

1. Set up the `.env` file.
2. Run the app (
   `cargo run --release`
   or
   `cargo run --release --features='<your-features>'`
   ).

> [!NOTE]
> Refer to the [`.env.example`](./.env.example) file for all the required
> variables and how to set them up accordingly.

### Advanced setting up (Containerization)

> [!IMPORTANT]
> Make sure you aren't running PostgreSQL, Jaeger or Ollama locally due to port
> conflicts!

The project uses Docker with compose. To run it just run:

```sh
docker-compose up -d
```

You need to install Docker Compose from
[docker.com/compose/install](https://docs.docker.com/compose/install/) though.

> [!NOTE]
> [`docker-compose.yml`](./docker-compose.yml) and the
> [`Dockerfile`](./Dockerfile) are set up for all the features.
