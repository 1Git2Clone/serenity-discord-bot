use crate::data::voicelines::HU_TAO_VOICELINES_JP;

use crate::prelude::*;

/// Send a Hu Tao quote!
#[poise::command(discard_spare_arguments, prefix_command, slash_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn quote(ctx: Context<'_>) -> Result<(), Error> {
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
