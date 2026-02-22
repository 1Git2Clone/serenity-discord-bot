FROM rust:1.92-slim-bullseye AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY serenity_discord_bot_derive/Cargo.toml serenity_discord_bot_derive/Cargo.toml

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir -p serenity_discord_bot_derive/src \
  && printf 'extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro]\npub fn dummy(_input: TokenStream) -> TokenStream { TokenStream::new() }\n' \
  > serenity_discord_bot_derive/src/lib.rs

RUN cargo build --release

RUN rm -rf src serenity_discord_bot_derive/src

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/serenity-discord-bot .
COPY .env .

CMD ["./serenity-discord-bot"]
