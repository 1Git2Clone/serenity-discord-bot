mod assets;
mod commands;
mod data;
mod database;
mod enums;
mod event_handler;
mod prelude;
mod tests;
mod utils;

use crate::prelude::*;
use event_handler::handler::event_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = START_TIME.elapsed().as_secs(); // Dummy data to get the time elapsing started

    let _ = dotenv::dotenv()?;

    // AI init happens in `setup` below, once the registered command list is known
    // (the system prompt is built from it).

    // With RUST_LOG unset, a bare "info" floods the console with every infra crate
    // (serenity, poise, h2, hyper, the gateway socket, …). Default to quieting them
    // and keeping the app at info; RUST_LOG still overrides when set.
    let env_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,serenity_discord_bot=info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_filter(env_layer);

    #[cfg(feature = "tokio_console")]
    let (console_layer, console_server) = console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .build();

    let registry = tracing_subscriber::registry().with(fmt_layer);

    #[cfg(feature = "tokio_console")]
    let registry = registry.with(console_layer);

    #[cfg(feature = "opentelemetry")]
    let registry = {
        let name = "Serenity Discord Bot";
        let resource = Resource::builder()
            .with_attributes(vec![
                KeyValue::new("service.name", name),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])
            .build();

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to create OTEL exporter");

        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();

        let tracer = tracer_provider.tracer(name);

        opentelemetry::global::set_tracer_provider(tracer_provider.clone());

        // Dedicated filter so the OTEL layer doesn't ingest the same infra firehose
        // (and the `tokio_unstable` runtime spans) and bury the app's traces.
        let otel_filter = EnvFilter::new("warn,serenity_discord_bot=info,tokio=off,runtime=off");

        registry.with(
            tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(otel_filter),
        )
    };

    registry.init();

    #[cfg(feature = "tokio_console")]
    tokio::spawn(console_server.serve());

    let token = BOT_TOKEN.to_string();
    // Either all or non_privileged intents only.
    // https://docs.rs/poise/latest/poise/#gateway-intents
    let intents = serenity::GatewayIntents::all() | serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { error, .. } => {
                            tracing::error!("Command failed: {error:?}");
                        }
                        err => poise::builtins::on_error(err).await.unwrap(),
                    }
                })
            },

            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                additional_prefixes: BOT_PREFIXES
                    .iter()
                    .map(|p| {
                        poise::Prefix::Regex(
                            RegexBuilder::new(p).case_insensitive(true).build().unwrap(),
                        )
                    })
                    .collect::<Vec<poise::Prefix>>(),
                mention_as_prefix: true,
                case_insensitive_commands: true,
                ignore_bots: true,
                ..Default::default()
            },
            commands: vec![
                commands::general_commands::help(),
                commands::general_commands::age(),
                commands::general_commands::reminder(),
                #[cfg(feature = "ai")]
                commands::general_commands::ai(),
                #[cfg(feature = "ai")]
                commands::general_commands::aichannel(),
                #[cfg(feature = "ai")]
                commands::general_commands::ai_review(),
                commands::general_commands::cookie(),
                commands::level_cmds::level(),
                commands::level_cmds::toplevels(),
                commands::embed_commands::tieup(),
                commands::embed_commands::pat(),
                commands::embed_commands::hug(),
                commands::embed_commands::kiss(),
                commands::embed_commands::slap(),
                commands::embed_commands::punch(),
                commands::embed_commands::bonk(),
                commands::embed_commands::nom(),
                commands::embed_commands::kill(),
                commands::embed_commands::kick(),
                commands::embed_commands::bury(),
                commands::embed_commands::selfbury(),
                commands::embed_commands::peek(),
                commands::embed_commands::avatar(),
                commands::embed_commands::drive(),
                commands::embed_commands::chair(),
                commands::embed_commands::boom(),
                commands::embed_commands::quote(),
                commands::embed_commands::uptime(),
                commands::embed_commands::custom(),
            ],
            manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let pool = Arc::new(connect_to_db().await.unwrap());

                #[cfg(feature = "ai")]
                {
                    use crate::data::{ai, cache};

                    // Self-healing command context: derive the list from what's
                    // actually registered so the system prompt never goes stale.
                    let commands: Vec<(String, String)> = framework
                        .options()
                        .commands
                        .iter()
                        .map(|cmd| {
                            (
                                cmd.name.clone(),
                                cmd.description.clone().unwrap_or_default(),
                            )
                        })
                        .collect();
                    ai::init_system_prompt(&commands);

                    // Build the provider now (fails fast on bad config) — after the
                    // system prompt is set so the command list is baked in.
                    LazyLock::force(&ai::AI_PROVIDER);
                    LazyLock::force(&ai::AI_MAX_MSG_CONTEXT);

                    ai::init_registered_channels(&pool).await?;
                    cache::init().await;
                }

                crate::data::custom_reactions::init(&pool).await?;

                tokio::spawn(crate::data::reminders::reminder_polling_loop(
                    Arc::clone(&ctx.http),
                    Arc::clone(&pool),
                ));

                Ok(Data {
                    // It's better to clone the bot user once when it starts rather than do http
                    // requests for the serenity::CurrentUser on every comman invocation.
                    bot_user: Arc::from(ready.user.clone()),
                    bot_avatar: Arc::<str>::from(ready.user.face().replace(".webp", ".png")),
                    available_commands: framework
                        .options()
                        .commands
                        .iter()
                        .map(|cmd| cmd.name.clone())
                        .collect(),
                    pool,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .activity(serenity::ActivityData::custom(format!(
            "Usable prefixes: [ {} ]",
            BOT_PREFIXES.join(", ")
        )))
        .status(serenity::OnlineStatus::Idle)
        .await?;

    // Multi-instance: when TOTAL_SHARDS / SHARD_START / SHARD_END are set,
    // each instance owns a disjoint range (both ends inclusive). Over-provision
    // the total (e.g. 16) and redistribute ranges to scale without resharding.
    if let (Ok(total), Ok(start), Ok(end)) = (
        std::env::var("TOTAL_SHARDS"),
        std::env::var("SHARD_START"),
        std::env::var("SHARD_END"),
    ) {
        let total: u32 = total
            .parse()
            .expect("TOTAL_SHARDS must be a positive integer");
        let start: u32 = start
            .parse()
            .expect("SHARD_START must be a non-negative integer");
        let end: u32 = end
            .parse()
            .expect("SHARD_END must be a non-negative integer");
        assert!(
            start <= end,
            "SHARD_START ({start}) must be <= SHARD_END ({end})"
        );
        assert!(
            end < total,
            "SHARD_END ({end}) is inclusive and must be < TOTAL_SHARDS ({total})"
        );
        tracing::info!("Starting shards {start}..={end} of {total}");
        // serenity's start_shard_range treats range.end as inclusive.
        client.start_shard_range(start..end, total).await?;
    } else {
        client.start().await?;
    }

    Ok(())
}
