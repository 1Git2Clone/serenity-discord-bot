# Deployment

How to run the bot: natively, with Docker Compose, with telemetry-only infra,
across multiple shards, and the blue-green topology used in production.

## Running locally

Copy `.env.example` to `.env`, fill it in, and make sure PostgreSQL is
reachable at `DATABASE_URL`. Migrations run automatically at startup.

```sh
cargo run --release
# or, with specific features:
cargo run --release --features='ai-deepseek opentelemetry'
```

`ai` is a meta-feature that also enables `redis`, and it needs exactly one
`ai-<backend>`. Building with `--features ai` alone fails with a
`compile_error!` by design. See the [README feature matrix](../README.md#features).

To run the telemetry stack (Grafana Tempo + Grafana) in containers while
running the bot natively:

```sh
docker-compose -f docker-compose.infra.yml up -d
```

## Docker Compose

The full compose file brings up PostgreSQL, Redis, Grafana Tempo, and Grafana
alongside the bot:

```sh
docker-compose up -d
```

Make sure you are not also running PostgreSQL or Grafana Tempo locally — the
ports conflict.

`docker-compose.infra.yml` is the infra-only variant: it brings up the
telemetry backends (Tempo + Grafana) without the bot, for when you run the bot
natively.

### Build features

The `Dockerfile` builds with the features listed in its `FEATURES` build arg,
which defaults to:

```
ai-deepseek opentelemetry tokio_console
```

Override it through the compose build args to change the provider or feature
set.

## Sharding and multi-instance

By default the bot runs as a single instance with one shard. To shard, set all
three of `TOTAL_SHARDS`, `SHARD_START`, and `SHARD_END`; set none of them to
stay single-instance. The instance then runs the shard range
`SHARD_START..=SHARD_END` out of `TOTAL_SHARDS` total.

The range is both-ends-inclusive. Asserts enforce `start <= end < total`.
(Serenity's `start_shard_range` treats the end as inclusive; the code bridges
Rust's exclusive `..`.)

The design intent is to over-provision `TOTAL_SHARDS` once — for example 16 —
and scale horizontally by redistributing disjoint ranges to new instances, with
no resharding. Two instances covering `0..=7` and `8..=15` of 16 total shards
together provide full coverage; both must be up.

Multi-instance deployments want `REDIS_URL` set, because the AI locks and rate
limits are coordination primitives that otherwise only hold per-process. See
the [Redis fallback model](./architecture.md#redis-optional-fallback-model).

## Production topology

The production deployment runs on a remote host. The 16 shards are split into
two instances — `a` covering shards 0–7 and `b` covering 8–15 — and both must
be up for full coverage.

Rollout is blue-green, driven by `scripts/bg-deploy.sh` through
[supervisor](http://supervisord.org/) (config under `deploy/supervisor/`),
using `sudo supervisorctl`. Each shard range has a blue/green pair so a new
build can be brought up and switched to without dropping gateway coverage.
