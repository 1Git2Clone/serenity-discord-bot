use super::*;

/// Displays the levels for the top 9 users.
#[poise::command(slash_command, prefix_command, rename = "toplevels")]
pub async fn toplevels(ctx: Context<'_>) -> Result<(), Error> {
    let message_guild_id = match ctx.guild_id() {
        Some(msg) => msg,
        None => {
            ctx.reply("Please use the fucking guild chats you sick fuck!")
                .await?;
            return Ok(());
        }
    };

    let db = connect_to_db(DATABASE_FILENAME.to_string()).await;

    let level_and_xp_rows = match db.await {
        Ok(pool) => fetch_top_nine_levels_in_guild(&pool, message_guild_id).await?,
        Err(_) => {
            ctx.reply("Please wait for the guild members to chat more.")
                .await?;
            return Ok(());
        }
    };
    ctx.defer().await?;
    let user_ids: Vec<u64> = level_and_xp_rows
        .par_iter()
        .map(|row| row.get::<i64, &str>(DATABASE_COLUMNS[&UserId]) as u64)
        .collect();
    let users = try_join_all(
        user_ids
            .iter()
            .map(|user_id| ctx.http().get_user((*user_id).into())),
    )
    .await?;

    let mut fields: Vec<(String, String, bool)> = Vec::new();

    for (counter, (row, user)) in level_and_xp_rows.iter().zip(users.iter()).enumerate() {
        let (level, xp) = (
            row.get::<i32, &str>(DATABASE_COLUMNS[&Level]),
            row.get::<i32, &str>(DATABASE_COLUMNS[&ExperiencePoints]),
        );

        fields.push((
            format!("#{} >> {}", counter + 1, user.name),
            format!(
                "Lvl: {}\nXP: {}\nLevel progress: {:.2}%",
                level,
                xp,
                ((xp as f32) / (level as f32))
            ),
            false,
        ));
    }

    let response = format!("Guild: {}\n\nTop 9 Users", ctx.guild().unwrap().name);
    let bot_user = Arc::clone(&ctx.data().bot_user);
    let bot_avatar = Arc::clone(&ctx.data().bot_avatar);

    let thumbnail = ctx
        .guild()
        .and_then(|guild| guild.icon_url())
        .unwrap_or_else(|| bot_avatar.to_string());

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title(response)
                .fields(fields)
                .thumbnail(thumbnail)
                .color((255, 0, 0))
                .footer(
                    serenity::CreateEmbedFooter::new(bot_user.tag())
                        .icon_url(bot_avatar.to_string()),
                ),
        ),
    )
    .await?;

    Ok(())
}
