FROM rust:1.94-bullseye AS builder
WORKDIR /app

COPY . .

ENV SQLX_OFFLINE=true
ARG RUSTFLAGS=""
ENV RUSTFLAGS=${RUSTFLAGS}

# Pick exactly one ai-<backend>. Override from compose to change provider/features.
ARG FEATURES="ai-deepseek opentelemetry tokio_console"
RUN cargo build --release --features "${FEATURES}"

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && update-ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/serenity-discord-bot .

CMD ["./serenity-discord-bot"]
