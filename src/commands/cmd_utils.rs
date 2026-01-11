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
pub async fn get_replied_user(ctx: Context<'_>) -> &serenity::User {
    let poise::Context::Prefix(msg_ctx) = ctx else {
        return ctx.author();
    };
    let ref_msg = msg_ctx.msg.referenced_message.as_deref();
    ref_msg.map_or(ctx.author(), |x| &x.author)
}

#[tracing::instrument(fields(category = "discord_command_utility",))]
pub fn same_user(u1: &User, u2: &User) -> bool {
    u1.id == u2.id
}

#[tracing::instrument(fields(category = "discord_command_utility",))]
pub fn get_rand_embed_from_type(embed_type: &EmbedType) -> Result<&'static str, Error> {
    let embed_option = COMMAND_EMBEDS[embed_type].choose(&mut rand::rng());
    match embed_option {
        Some(embed) => Ok(embed),
        None => {
            Err("Failed to get item from the matching vector of strings from the Hash Map.".into())
        }
    }
}

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command_utility",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        target_user = %target_replied_user.id.get(),
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        embed = ?embed
    )
)]
pub async fn make_full_response(
    ctx: &Context<'_>,
    target_replied_user: &serenity::User,
    embed: Option<serenity::CreateEmbed>,
) -> poise::CreateReply {
    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
    };

    let mut reply = poise::CreateReply::default().content(match ping_on_shash_command {
        Some(ping) => format!("{}", ping),
        None => "".into(),
    });

    if let Some(e) = embed {
        reply = reply.embed(e);
    };

    reply
}
