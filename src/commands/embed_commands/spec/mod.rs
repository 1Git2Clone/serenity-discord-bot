//! The shared implementation behind the embed (GIF) commands.
//!
//! Each interaction command (hug, slap, ...) is a thin wrapper passing an
//! [`InteractionSpec`] to [`run_interaction`]; the simpler no-target commands
//! go through [`send_embed`].

use super::user_interaction;
use crate::prelude::*;

pub mod interactions;
pub mod solo;

/// How an interaction command behaves; one spec per command.
pub struct InteractionSpec {
    /// The embed type the random GIF is picked from.
    pub embed: EmbedType,
    /// The verb in the "**author** *verb* **target**" embed title.
    pub verb: &'static str,
    /// What to do when the author targets themselves.
    pub on_self: OnSelf,
    /// Extra plain reply sent after the self-target response.
    pub self_followup: Option<fn(&serenity::User) -> String>,
}

/// Self-target behavior. Every `fn` receives the invoking user.
pub enum OnSelf {
    /// Send the command's embed with this content.
    Embed(fn(&serenity::User) -> String),
    /// Send an embed of a different type with this content.
    EmbedAs(EmbedType, fn(&serenity::User) -> String),
    /// Send plain text, no embed.
    Text(fn(&serenity::User) -> String),
    /// Don't fall back to the replied-to message's author: without an
    /// explicit target, reply with this message instead.
    RequireTarget(fn(&serenity::User) -> String),
}

/// The shared red embed with the bot tag footer.
fn footer_embed(ctx: &Context<'_>, image: String) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .color((255, 0, 0))
        .image(image)
        .footer(
            serenity::CreateEmbedFooter::new(ctx.data().bot_user.tag())
                .icon_url(ctx.data().bot_avatar.to_string()),
        )
}

/// Send a footer embed with an optional title; the body of the no-target
/// commands (selfbury, peek, drive, chair, boom).
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
async fn send_embed(ctx: Context<'_>, title: Option<String>, image: String) -> Result<(), Error> {
    let mut embed = footer_embed(&ctx, image);
    if let Some(title) = title {
        embed = embed.title(title);
    }
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Resolve the target, handle the self-target case per the spec, and send the
/// "**author** *verb* **target**" embed.
#[tracing::instrument(
    skip(ctx, spec),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        target_user = %user.as_ref().map(|u| u.id.get()).unwrap_or(0),
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
async fn run_interaction(
    ctx: Context<'_>,
    user: Option<serenity::User>,
    spec: InteractionSpec,
) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&spec.embed)?;

    let target = if let OnSelf::RequireTarget(msg) = &spec.on_self {
        let Some(target) = user.as_ref() else {
            ctx.send(poise::CreateReply::default().content(msg(ctx.author())))
                .await?;
            return Ok(());
        };
        target
    } else {
        let target = user.as_ref().unwrap_or(get_replied_user(ctx).await);
        if same_user(target, ctx.author()) {
            match &spec.on_self {
                OnSelf::Embed(msg) => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(msg(target))
                            .embed(footer_embed(&ctx, embed_item.to_string())),
                    )
                    .await?;
                }
                OnSelf::EmbedAs(embed_type, msg) => {
                    let image = cmd_utils::get_rand_embed_from_type(embed_type)?.to_string();
                    ctx.send(
                        poise::CreateReply::default()
                            .content(msg(target))
                            .embed(footer_embed(&ctx, image)),
                    )
                    .await?;
                }
                OnSelf::Text(msg) => {
                    ctx.send(poise::CreateReply::default().content(msg(target)))
                        .await?;
                }
                // Handled by the outer `if let`.
                OnSelf::RequireTarget(_) => unreachable!(),
            }
            if let Some(followup) = spec.self_followup {
                ctx.reply(followup(target)).await?;
            }
            return Ok(());
        }
        target
    };

    let response: String = user_interaction(&ctx, ctx.guild_id(), ctx.author(), target, |u1, u2| {
        format!("**{u1}** *{}* **{u2}**", spec.verb)
    })
    .await;

    let embed = footer_embed(&ctx, embed_item.to_string()).title(response);
    let full_response = make_full_response(&ctx, target, Some(embed)).await;
    ctx.send(full_response).await?;

    Ok(())
}
