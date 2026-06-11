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

# git + gh are runtime dependencies of /ai-review (the default FEATURES
# include ai-deepseek). gh isn't in the Debian repos, so it comes from
# GitHub's apt repo.
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl git \
  && update-ca-certificates \
  && curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
  && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    > /etc/apt/sources.list.d/github-cli.list \
  && apt-get update \
  && apt-get install -y --no-install-recommends gh \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/serenity-discord-bot .

CMD ["./serenity-discord-bot"]
