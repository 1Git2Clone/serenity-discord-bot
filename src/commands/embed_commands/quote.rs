use rand::seq::SliceRandom;

use crate::data::bot_data::HU_TAO_VOICELINES_JP;

use super::*;

/// Send a peek GIF in the chat (you lurker)
#[poise::command(prefix_command, slash_command)]
pub async fn quote(ctx: Context<'_>) -> Result<(), Error> {
    let response = HU_TAO_VOICELINES_JP
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string();

    ctx.send(poise::CreateReply::default().content(response))
        .await?;

    Ok(())
}
