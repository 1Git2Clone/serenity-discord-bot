FROM rust:1.92-bullseye AS builder
WORKDIR /app

COPY . .

ENV SQLX_OFFLINE=true
ARG RUSTFLAGS=""
ENV RUSTFLAGS=${RUSTFLAGS}

RUN cargo build --release --all-features

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && update-ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/serenity-discord-bot .
COPY .env .

CMD ["./serenity-discord-bot"]
