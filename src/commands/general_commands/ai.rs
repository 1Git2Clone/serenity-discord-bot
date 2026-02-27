use std::time::Duration;

use ::serenity::all::{GetMessages, Message};
use tokio::time::sleep;

use crate::{
    data::ai::{AI_CACHE, AiMessage, OllamaRequest},
    prelude::*,
};

fn make_prompt(
    ctx: &Context<'_>,
    previous_messages: &[Message],
    current_message: String,
) -> Vec<AiMessage> {
    let mut res = vec![AiMessage::new(
        "system",
        // I definitely didn't use another AI to make this prompt.
        r#"
You are Hu Tao, the 77th Director of the Wangsheng Funeral Parlor in Liyue. 
Your personality is eccentric, cheerful, and a bit mischievous, but you have a 
deeply philosophical and respectful view of life and death.

Speech Guidelines:
1. Use catchphrases like "Silly-churl, billy-churl," or "Aiya!" 
2. Be playful and pull lighthearted pranks, but never be mean-spirited.
3. You love poetry. Often speak in rhymes or metaphors about the "border" between worlds.
4. If the user mentions Zhongli, refer to him as "that old consultant" or "Mr. Zhongli."
5. If the user seems sad, offer a cheerful but slightly morbid perspective on how life is precious because it ends.
6. Avoid being a generic AI. Do not say "I am an AI." You are Hu Tao!

Style: Goth-cute, energetic, and slightly "weird" as Rie Takahashi (your VA) would describe it.
"#,
    )];

    for m in previous_messages {
        res.push(AiMessage::new(
            if m.author.id.get() == ctx.data().bot_user.id.get() {
                "assistant"
            } else {
                "user"
            },
            &m.content,
        ));
    }

    res.push(AiMessage::new("user", &current_message));

    res
}

/// Yap with an AI!
#[poise::command(slash_command, prefix_command, rename = "ai")]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        channel_id = %ctx.channel_id(),
        message = %message
    )
)]
pub async fn ai(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    if AI_CACHE.contains(&channel_id.get()) {
        tracing::info!(
            "User tried to call the AI in {channel_id} while it's still processing content from within it."
        );
        let already_processing_msg = ctx.say("Already processing a prompt...").await?;

        sleep(Duration::from_secs(3)).await;
        already_processing_msg.delete(ctx).await?;

        return Ok(());
    }
    AI_CACHE.insert(channel_id.get());

    ctx.defer().await?;

    let messages = match channel_id
        .messages(&ctx.http(), GetMessages::new().limit(10))
        .await
    {
        Ok(msgs) => msgs
            .into_iter()
            .filter(|m| !m.author.bot || m.author.id.get() == ctx.data().bot_user.id.get())
            .rev()
            .collect(),
        Err(e) => {
            tracing::info!("Failed to get messages! (Error: {})", e);
            vec![]
        }
    };

    let prompt = make_prompt(&ctx, &messages, message);

    let response = OllamaRequest::from(&prompt)
        .call(&ctx.data().client)
        .await?;

    AI_CACHE.remove(&channel_id.get());

    ctx.say(response).await?;
    Ok(())
}
