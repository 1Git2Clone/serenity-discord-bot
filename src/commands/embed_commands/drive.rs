use super::*;

/// Get a Ryan Gosling drive GIF.
#[poise::command(slash_command, prefix_command, rename = "drive")]
pub async fn drive(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::RyanGoslingDrive).await?;

    let embed = serenity::CreateEmbed::new()
        // .title()
        .color((255, 0, 0))
        .image(embed_item);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
