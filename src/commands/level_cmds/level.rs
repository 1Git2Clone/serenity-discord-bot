use super::*;

/// Displays the user's level
#[poise::command(slash_command, prefix_command, rename = "level")]
pub async fn level(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let message_guild_id = match ctx.guild_id() {
        Some(msg) => msg,
        None => {
            ctx.reply("Please use the fucking guild chats you sick fuck!")
                .await?;
            return Ok(());
        }
    };

    let target_replied_user = user.as_ref().unwrap_or(get_replied_user(ctx).await);
    let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
    let level_xp_and_rank_row_option = match db.await {
        Ok(pool) => {
            println!("Connected to the database: {pool:?}");
            fetch_user_level_and_rank(&pool, target_replied_user, message_guild_id).await?
        }
        Err(_) => {
            ctx.reply(format!(
                "Please wait for {} to chat more then try again later...",
                target_replied_user.name
            ))
            .await?;
            return Ok(());
        }
    };
    let level_xp_and_rank_row = if let Some(lvl_xp_and_rank_row) = level_xp_and_rank_row_option {
        lvl_xp_and_rank_row
    } else {
        ctx.reply(format!(
            "Please wait for {} to chat more then try again later...",
            target_replied_user.name
        ))
        .await?;
        return Ok(());
    };
    let level = level_xp_and_rank_row
        .1
        .get::<i32, &str>(DATABASE_COLUMNS[&Level]);
    let xp = level_xp_and_rank_row
        .1
        .get::<i32, &str>(DATABASE_COLUMNS[&ExperiencePoints]);

    let avatar = target_replied_user.face().replace(".webp", ".png");
    let username = &target_replied_user.name;
    let response = format!(
        "User stats for: **{}**\n\nRank: {}",
        &username, level_xp_and_rank_row.0
    );
    let bot_user = Arc::clone(&ctx.data().bot_user);
    let bot_avatar = Arc::clone(&ctx.data().bot_avatar).to_string();
    let percent_left_to_level_up: f32 = (xp as f32) / (level as f32);
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
                    format!("⊱ {:.2}%", percent_left_to_level_up),
                    false,
                )
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;
    Ok(())
}
