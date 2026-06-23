# Observability

The bot builds a layered `tracing_subscriber` in `main.rs`. The layers are a
console formatter, an optional Tokio Console layer, and an optional
OpenTelemetry OTLP exporter — each with its own filter.

## Console logging

The console formatter's default filter is `warn,serenity_discord_bot=info`.
With `RUST_LOG` unset, the infra crates (serenity, h2, hyper, the gateway) are
quieted to `warn` so the bot's own `info` logs are visible. Setting `RUST_LOG`
overrides the default entirely.

## The `category` span field

Every instrumented span carries a custom `category` field for filtering. Values
include `redis`, `ai_chat`, `llm`, `sql`, and `discord_command`.

The top-level `event_handler` is deliberately not instrumented: it fires for
every gateway event, including presence updates, so a span there would be pure
noise. Only handled events carry spans.

## Tokio Console

Task-level async runtime inspection through
[Tokio Console](https://github.com/tokio-rs/console). It is feature-gated and
needs the `tokio_unstable` cfg at build time:

```sh
RUSTFLAGS="--cfg tokio_unstable" cargo run --features tokio_console
```

![tokio-console task view](../assets/tokio-console-demo.png)

## OpenTelemetry

The OTLP layer is feature-gated behind `opentelemetry` and exports over
gRPC/tonic, so it can point at any OTLP-compatible collector. It uses a
separate filter from the console layer:

```
warn,serenity_discord_bot=info,tokio=off,runtime=off
```

`tokio` and `runtime` are turned off here so that, when `tokio_unstable` is
also enabled, the runtime-span firehose does not bury application traces in the
trace backend.

The compose stack ships [Grafana Tempo](https://grafana.com/oss/tempo/) and
Grafana pre-wired as the UI. To bring up the telemetry backends without the bot:

```sh
docker-compose -f docker-compose.infra.yml up -d
```

To run Tempo manually, create `/var/tempo` once with your user as owner:

```sh
sudo mkdir -p /var/tempo && sudo chown $USER /var/tempo
tempo -config.file=./tempo.yaml
```

![otel-tui trace view](../assets/otel-tui-trace.png)

![Grafana Tempo trace view](../assets/grafana-tempo-trace.png)

![Grafana Tempo flame graph](../assets/grafana-tempo-flamegraph.png)

![Grafana Tempo span details](../assets/grafana-tempo-span-details.png)
