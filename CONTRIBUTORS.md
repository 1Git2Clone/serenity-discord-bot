# Contributing

## Development environment

- Rust toolchain, edition 2024 (rustc 1.94 or newer — see `rust-version` in
  `Cargo.toml`).
- PostgreSQL, reachable at `DATABASE_URL`. Migrations run automatically at
  startup.
- Redis, optionally. The bot runs fine without it; every Redis-backed feature
  has a single-instance fallback. See the
  [Redis fallback model](./docs/architecture.md#redis-optional-fallback-model).

Copy `.env.example` to `.env` and fill in the values. See
[docs/configuration.md](./docs/configuration.md) for the full variable list.

## Building with features

Everything optional is a Cargo feature, and `ai` is a meta-feature that also
enables `redis`.

```sh
# Core only:
cargo run --release

# With AI — pick exactly one backend:
cargo run --release --features ai-deepseek
```

`ai` does not pick a provider. You must enable exactly one of `ai-deepseek`,
`ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, or `ai-groq`. Building
with `--features ai` and no backend stops at a `compile_error!`:

```
The `ai` feature needs a backend. Enable exactly one of: `ai-deepseek`,
`ai-ollama`, `ai-anthropic`, `ai-openai`, `ai-google`, `ai-groq`.
```

Tokio Console needs the `tokio_unstable` cfg at build time:

```sh
RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio_console
```

See the [README feature matrix](./README.md#features) for the full list.

## sqlx offline

Queries are checked against the database schema at compile time. The committed
`.sqlx/` directory holds the offline query cache, so the project builds without
a live database.

When you add or change a query, regenerate the cache against a live database:

```sh
cargo sqlx prepare
```

Commit the updated `.sqlx/` files alongside the query change.

## Migrations are append-only

Never edit or reorder a migration that has already been applied anywhere. To
change the schema, add a new additive migration.

## Tests

```sh
cargo test
```

Tests that need Redis use a `test_redis()` helper and skip silently when no
Redis is reachable, so the suite passes without infrastructure.

Coverage is generated through `scripts/coverage.sh` (lcov; a Codecov badge is
wired up in the README). CI runs on both GitHub Actions
(`.github/workflows/full.yml`) and GitLab (`.gitlab-ci.yml`).

## Git hooks

Install the repository's pre-commit hook:

```sh
scripts/setup-hooks.sh
```

## Commit conventions

Conventional commits: `type(scope): short description`. Before committing, scan
recent history for the established scope names and match them — do not invent
new scopes without checking:

```sh
git log --oneline -50
```

Append a co-author trailer for tool-assisted commits. See [AGENTS.md](./AGENTS.md)
for the exact format.

## Code style

Read [AGENTS.md](./AGENTS.md): KISS, YAGNI, and surgical changes — touch only
what the task requires, and match existing style. Doc comments follow the same
register as this documentation: plain, terse, declarative. No marketing
adjectives, no emoji, no bold-for-emphasis sprinkled through prose.

## The derive crate

`serenity_discord_bot_derive/` is a local proc-macro crate (path dependency)
with four small derives — deduplication through shared helper functions over a
spec, not a macro DSL:

- `IterateVariants` — adds `fn variants() -> &'static [Self]`.
- `DiscordEmoji` (attribute `emoji_id`) — `Display` renders `<:Name:id>`, plus
  `get_id()` and `get_variant_str()`.
- `Asset` (attributes `base_url`, `src_path`) — `Display` yields
  `base_url/src_path`, used for CDN asset URLs.
- `DatabaseEnum` — `Display` converts `PascalCase` variants to `snake_case`,
  plus an `.as_str()`.

See [docs/architecture.md](./docs/architecture.md) for the broader picture.
