# Documentation

Project documentation for serenity-discord-bot. Start with the top-level
[README](../README.md) for what the bot is and how to run a minimal version.

- [architecture.md](./architecture.md) — crate layout, startup sequence,
  message data flow, the two AI paths, the tool registry, and the
  Redis-optional fallback model.
- [configuration.md](./configuration.md) — the full environment-variable
  reference: what each one does, defaults, which feature it belongs to, and
  which are secret.
- [deployment.md](./deployment.md) — running natively, Docker Compose,
  telemetry-only infra, sharding and multi-instance, and the blue-green
  production topology.
- [ai.md](./ai.md) — enabling AI, the persona chat and context window, and a
  full `/ai-review` setup and usage walkthrough.
- [custom-reactions.md](./custom-reactions.md) — the per-guild numbering model,
  safe removal, image-URL validation, and the write-through Redis cache.
- [observability.md](./observability.md) — tracing layers, the `category` span
  field, Tokio Console, and OpenTelemetry with Tempo/Grafana.

See also [SECURITY.md](../SECURITY.md) for the secrets inventory and the AI
code review threat model, and [CONTRIBUTORS.md](../CONTRIBUTORS.md) for the dev
environment and workflow.
