use crate::prelude::*;

const LEVEL_STEPS: [f32; 14] = [
    7.14, 14.28, 21.41, 28.56, 35.69, 42.83, 49.98, 57.12, 64.25, 71.39, 78.53, 85.67, 92.82, 99.96,
];

fn chat_more(username: &str) -> String {
    format!(
        "Please wait for {} to chat more then try again later...",
        username
    )
}

/// Displays the user's level
#[poise::command(slash_command, prefix_command)]
pub async fn level(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let Some(message_guild_id) = ctx.guild_id() else {
        ctx.reply("Only works in guilds!").await?;
        return Ok(());
    };

    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
    let Ok(pool) = db else {
        ctx.reply(chat_more(&target_replied_user.name)).await?;
        return Ok(());
    };
    let level_xp_and_rank_row_option =
        fetch_user_level_and_rank(&pool, target_replied_user, message_guild_id).await?;

    let Some(level_xp_and_rank_row) = level_xp_and_rank_row_option else {
        ctx.reply(chat_more(&target_replied_user.name)).await?;
        return Ok(());
    };
    let level = level_xp_and_rank_row
        .1
        .get::<i32, &str>(LEVELS_TABLE[&LevelsSchema::Level]);
    let xp = level_xp_and_rank_row
        .1
        .get::<i32, &str>(LEVELS_TABLE[&LevelsSchema::ExperiencePoints]);

    let avatar = target_replied_user.face().replace(".webp", ".png");
    let username = &target_replied_user.name;
    let response = format!(
        "User stats for: **{}**\n\nRank: {}",
        &username, level_xp_and_rank_row.0
    );
    let bot_user = Arc::clone(&ctx.data().bot_user);
    let bot_avatar = Arc::clone(&ctx.data().bot_avatar).to_string();
    let percent_left_to_level_up: f32 = (xp as f32) / (level as f32);
    let progress_bar: String = {
        LEVEL_STEPS
            .iter()
            .map(|x| {
                if percent_left_to_level_up > *x {
                    return "█";
                }
                "▒"
            })
            .collect()
    };
    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title(response)
                .url("")
                .color((255, 0, 0))
                .thumbnail(&avatar)
                .field("Level", format!("⊱ {}", level), false)
                .field("Experience Points", format!("⊱ {}", xp), false)
                .field(
                    "Progress until next level",
                    format!("┊{}┊\n╰• {:.2}%", progress_bar, percent_left_to_level_up),
                    false,
                )
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;
    Ok(())
}
