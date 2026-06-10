//! The user interaction commands, each a spec passed to [`run_interaction`].

use super::{InteractionSpec, OnSelf, run_interaction};
use crate::prelude::*;

/// Tie someone up (HUH?)
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn tieup(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::TieUp,
            verb: "ties up",
            on_self: OnSelf::Embed(|_| "Y'know what? Sure, I'll tie you up!".to_string()),
            self_followup: Some(|target| {
                format!(
                    "Did you like it {}? You filthy degenerate~",
                    target.mention()
                )
            }),
        },
    )
    .await
}

/// Pat someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn pat(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Pat,
            verb: "pats",
            on_self: OnSelf::Embed(|_| "Aww~ I'll pat you!".to_string()),
            self_followup: None,
        },
    )
    .await
}

/// Hug someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn hug(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Hug,
            verb: "hugs",
            on_self: OnSelf::Embed(|_| "Aww~ I'll hug you!".to_string()),
            self_followup: None,
        },
    )
    .await
}

/// Kiss someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn kiss(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Kiss,
            verb: "kisses",
            on_self: OnSelf::EmbedAs(EmbedType::Slap, |_| {
                "Aww~ I won't kiss you! Ahahahah!".to_string()
            }),
            self_followup: None,
        },
    )
    .await
}

/// Slap someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn slap(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Slap,
            verb: "slaps",
            on_self: OnSelf::Embed(|_| "Why do you want to get slapped??".to_string()),
            self_followup: Some(|target| format!("Did you like it? {}", target.mention())),
        },
    )
    .await
}

/// Punch someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn punch(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Punch,
            verb: "punches",
            on_self: OnSelf::Text(|_| "I won't punch you! *pouts*".to_string()),
            self_followup: None,
        },
    )
    .await
}

/// Bonk someone who's horknee
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn bonk(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Bonk,
            verb: "bonks",
            on_self: OnSelf::Embed(|_| "バカ！".to_string()),
            self_followup: None,
        },
    )
    .await
}

/// Nom someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn nom(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Nom,
            verb: "noms",
            on_self: OnSelf::Embed(|target| format!("{} noms themselves...?", target.name)),
            self_followup: None,
        },
    )
    .await
}

/// Kill someone (Sadge)
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn kill(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Kill,
            verb: "kills",
            on_self: OnSelf::Text(|_| "No.".to_string()),
            self_followup: None,
        },
    )
    .await
}

/// Kick someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Kick,
            verb: "kicks",
            on_self: OnSelf::Text(|target| {
                format!(
                    "{}, why would you kick yourself...? Weirdo...",
                    target.mention()
                )
            }),
            self_followup: None,
        },
    )
    .await
}

/// Bury someone
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
pub async fn bury(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    run_interaction(
        ctx,
        user,
        InteractionSpec {
            embed: EmbedType::Bury,
            verb: "buries",
            on_self: OnSelf::RequireTarget(|author| {
                format!(
                    "{} Just use the `!selfbury` or `/selfbury` command bruh...",
                    author.mention()
                )
            }),
            self_followup: None,
        },
    )
    .await
}
