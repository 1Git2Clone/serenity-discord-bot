mod assets;
mod commands;
mod data;
mod database;
mod enums;
mod event_handler;
mod extra_threads;
mod prelude;
mod tests;
mod utils;

use crate::prelude::*;
use event_handler::handler::event_handler;
use extra_threads::xp_command_cooldown::periodically_clean_users_on_diff_thread;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = START_TIME.elapsed().as_secs(); // Dummy data to get the time elapsing started

    let _ = dotenv::dotenv()?;

    let env_layer = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_filter(env_layer);

    #[cfg(feature = "tokio_console")]
    let console_layer = console_subscriber::spawn();

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

        registry.with(tracing_opentelemetry::layer().with_tracer(tracer))
    };

    registry.init();

    let token = BOT_TOKEN.to_string();
    periodically_clean_users_on_diff_thread();
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
                        poise::FrameworkError::Command { ctx, .. } => {
                            tracing::info!(
                                "In on_error: {:?}",
                                ctx.invocation_data::<&str>().await.as_deref()
                            );
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
            ],
            manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
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
                    pool: Arc::new(connect_to_db().await.unwrap()),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .activity(serenity::ActivityData::custom(format!(
            "Usable prefixes: [ {} ]",
            BOT_PREFIXES.join(", ")
        )))
        .status(serenity::OnlineStatus::Idle)
        .await;

    client?.start().await?;

    Ok(())
}
