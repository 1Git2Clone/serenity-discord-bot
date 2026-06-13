use crate::prelude::*;

pub mod spec;
pub use spec::custom_reaction::custom;
pub use spec::interactions::*;
pub use spec::solo::*;

pub mod quote;
pub use quote::quote;

pub mod uptime;
pub use uptime::uptime;

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
    skip(ctx, f),
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
    f: impl Fn(&str, &str) -> String,
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
