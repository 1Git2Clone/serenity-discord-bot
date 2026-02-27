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
- A bunch of embed interaction commands (like pats, hugs and etc.)
- A levelling system (managed locally). Some people like it, others hate it and
  say its a MEE6 clone (They kinda have a point there). Nevertheless, you can
  always use the Postgres database for something else if you need to. There's also
  a cooldown implementation and periodic cleaning up of the users which aren't
  under cooldown (This was a bit tricky to do because of the Mutex locking and my
  overall skill issue).
- The leveling system has a nice topranks command which gives a cool-looking embed!
- A bot uptime command.

### Optional features

#### AI

There's an optional `ai` feature (which can be enabled with `--features="ai"`)
using [ollama](https://docs.ollama.com/quickstart).

To use it you simply need to run:

```sh
ollama pull gwen2.5:1.5b # <- You can use any model.
ollama serve
```

> [!NOTE]
> You can use any model you like, just make sure to set it in the
> [`src/data/ai.rs`](./src/data/ai.rs) at
> `crate::data::ai::OllamaRequest::DEFAULT_MODEL`.

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

> [!IMPORTANT]
> **ALL THE RELATIVE PATHS IN THIS DESCRIPTION BASED ON THE ROOT OF THE REPOSITORY!**

In order to set up your bot token you can do it manually by creating and
modifying the `.env` file and adding the token in it like this:

```env
BOT_TOKEN=YOUR_DISCORD_BOT_TOKEN_HERE
```

Alternatively, there's also a script that does the exact same thing! To compile
and run it, enter in the following commands:

For Linux;

```sh
cd extra_utils/generate_dotenv
cargo build
cd .. # <- project root.
./path/to/bin # <- default: `./extra_utils/generate_dotenv/target/<debug|release>/generate_dotenv`
```

For Windows; Just change `/` to `\`).

After you've configured your `dotenv` (`.env`) files, you run:

```sh
cargo install sqlx-cli

sqlx database setup

sqlx migrate run

cargo sqlx prepare
```

For the first time only, and then just:

```sh
cargo run --release
```

For any following iteration.

### Advanced setting up (Containerization)

> [!NOTE]
> Set up the sqlx migration and your PostgreSQL database before using Docker.

The project uses Docker and Docker Compose in order to set up the PostgreSQL
database connection and run the bot with it.

> [!IMPORTANT]
> In case you do use containerization, make sure to connect to
> `@172.17.0.1`/`@host.docker.internal` instead of `localhost` as you'll get
> connection errors.

```sh
docker-compose up -d
```

You need to install Docker Compose from
[docker.com/compose/install](https://docs.docker.com/compose/install/) though.

## Additional info

If you want to change your database location, credentials, host, port, path or
network, make sure you also change the `.env` file.

```env
DATABASE_URL="postgres://<user>:<password>@<network>:<port>/bot_database"
```

NOTE: The database URL configuration is hard-coded in the
`extra_utils/generate_dotenv.rs` file.
