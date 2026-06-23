# Configuration

All configuration is through environment variables, read from `.env` at
startup (via `dotenv`) or from the process environment. Copy `.env.example` to
`.env` and fill in the values.

Migrations run automatically at startup, so a reachable `DATABASE_URL` is
brought up to date without a manual step.

## Environment variables

"Required" means the process — or the relevant feature — panics or refuses to
operate without the variable. Variables scoped to a feature only matter when
that feature is compiled in.

| Var | Scope | Required? | Notes |
|---|---|---|---|
| `BOT_TOKEN` | core | yes | Discord bot token. Secret. |
| `DATABASE_URL` | core | yes | Postgres DSN. Migrations auto-run. |
| `DB_USER` / `DB_PASSWORD` / `DB_NAME` / `DB_NETWORK` / `DB_PORT` | core | compose | Composed into `DATABASE_URL` for Docker Compose. |
| `RUST_LOG` | core | no | Overrides the default tracing filter. |
| `AI_MODEL` | ai | yes (if AI built) | Panics if unset when AI is compiled in. |
| `AI_API_KEY` | ai (hosted) | yes for hosted backends | Not needed for `ai-ollama`. Secret. |
| `AI_MAX_MSG_CONTEXT` | ai | no (default 10, cap 100) | Context window size. |
| `AI_CHAT_ENDPOINT` | ai-ollama only | no | Override the Ollama base URL. |
| `REDIS_URL` | redis / ai | no | Enables cross-instance context, locks, and limits. |
| `TOTAL_SHARDS` / `SHARD_START` / `SHARD_END` | sharding | no | All three or none. Inclusive range. |

## Secrets

`BOT_TOKEN` and `AI_API_KEY` are secrets. None of these are ever logged. See
[SECURITY.md](../SECURITY.md) for the full inventory and handling.

## Feature-specific notes

- `ai` is a meta-feature: enabling it also enables `redis`. It does not pick a
  provider — you must enable exactly one `ai-<backend>`. See the
  [README feature matrix](../README.md#features) and [docs/ai.md](./ai.md).
- `util-download` needs `yt-dlp` and `ffmpeg` (with `ffprobe`) on `PATH` at
  runtime. No additional environment variables.
- `tokio_console` additionally needs `RUSTFLAGS="--cfg tokio_unstable"` at
  build time. See [docs/observability.md](./observability.md).
- Sharding is opt-in: set all three of `TOTAL_SHARDS`, `SHARD_START`, and
  `SHARD_END`, or none. See [docs/deployment.md](./deployment.md).
