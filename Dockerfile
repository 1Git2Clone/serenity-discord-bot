FROM rust:1.92-slim-bullseye AS builder
WORKDIR /app

COPY . .
# ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/serenity-discord-bot .
COPY .env .

CMD ["./serenity-discord-bot"]
