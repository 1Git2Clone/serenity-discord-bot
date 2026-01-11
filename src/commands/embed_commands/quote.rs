use crate::data::voicelines::HU_TAO_VOICELINES_JP;

use crate::prelude::*;

/// Send a Hu Tao quote!
#[poise::command(prefix_command, slash_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        extra_msg = %msg.as_deref().unwrap_or("")
    )
)]
pub async fn quote(ctx: Context<'_>, #[rest] msg: Option<String>) -> Result<(), Error> {
    let response = HU_TAO_VOICELINES_JP
        .choose(&mut rand::rng())
        .ok_or("No Hu Tao voicelines!")?
        .to_string();
    let emoji = Emojis::variants()
        .choose(&mut rand::rng())
        .ok_or("No Emoji Variants!")?
        .to_string();

    ctx.send(poise::CreateReply::default().content([response, emoji].join(" ")))
        .await?;

    Ok(())
}
