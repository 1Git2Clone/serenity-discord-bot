use super::*;

/// Get a Ryan Gosling drive GIF.
#[poise::command(slash_command, prefix_command)]
pub async fn drive(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::RyanGoslingDrive)?;

    let embed = serenity::CreateEmbed::new()
        .color((255, 0, 0))
        .image(embed_item);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
