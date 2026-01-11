use crate::prelude::*;

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command_utility",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn get_name(ctx: &Context<'_>, guild_id: Option<GuildId>, u: &serenity::User) -> String {
    let base_case = || u.name.clone();
    match guild_id {
        Some(id) => u.nick_in(ctx, id).await.unwrap_or(base_case()),
        None => base_case(),
    }
}

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command_utility",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        user_1_id = %user_1.id,
        user_2_id = %user_2.id,
    )
)]
pub async fn user_interaction(
    ctx: &Context<'_>,
    guild_id: Option<GuildId>,
    user_1: &serenity::User,
    user_2: &serenity::User,
    f: fn(name_1: &str, name_2: &str) -> String,
) -> String {
    let [ref name_1, ref name_2] = join_all([
        get_name(ctx, guild_id, user_1),
        get_name(ctx, guild_id, user_2),
    ])
    .await[..] else {
        return String::from("");
    };

    f(name_1, name_2)
}

// #region User interaction commands

pub mod tieup;
pub use tieup::tieup;

pub mod pat;
pub use pat::pat;

pub mod hug;
pub use hug::hug;

pub mod kiss;
pub use kiss::kiss;

pub mod slap;
pub use slap::slap;

pub mod punch;
pub use punch::punch;

pub mod bonk;
pub use bonk::bonk;

pub mod nom;
pub use nom::nom;

pub mod kill;
pub use kill::kill;

pub mod kick;
pub use kick::kick;

pub mod bury;
pub use bury::bury;

pub mod selfbury;
pub use selfbury::selfbury;

pub mod peek;
pub use peek::peek;

// #endregion

pub mod avatar;
pub use avatar::avatar;

pub mod drive;
pub use drive::drive;

pub mod chair;
pub use chair::chair;

pub mod boom;
pub use boom::boom;

pub mod quote;
pub use quote::quote;

pub mod uptime;
pub use uptime::uptime;
