use rand::seq::SliceRandom;

use crate::data::bot_data::HU_TAO_VOICELINES_JP;

use crate::prelude::*;

/// Send a Hu Tao quote!
#[poise::command(prefix_command, slash_command)]
pub async fn quote(ctx: Context<'_>, #[rest] _msg: Option<String>) -> Result<(), Error> {
    let response = HU_TAO_VOICELINES_JP
        .choose(&mut rand::thread_rng())
        .ok_or("No Hu Tao voicelines!")?
        .to_string();

    ctx.send(poise::CreateReply::default().content(response))
        .await?;

    Ok(())
}
