use crate::commands::level_logic::calculate_xp_to_level_up;

use crate::prelude::*;

/// Displays the levels for the top 9 users.
#[poise::command(slash_command, prefix_command)]
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
pub async fn toplevels(ctx: Context<'_>, #[rest] msg: Option<String>) -> Result<(), Error> {
    let Some(message_guild_id) = ctx.guild_id() else {
        ctx.reply("This command only works in guilds!").await?;
        return Ok(());
    };

    let user_xp_lvl =
        LevelsTable::fetch_top_nine_users(&ctx.data().pool, message_guild_id.into()).await?;

    ctx.defer().await?;

    let user_ids: Vec<i64> = user_xp_lvl.iter().map(|row| row.user_id).collect();
    let users = try_join_all(
        user_ids
            .iter()
            .map(|user_id| ctx.http().get_user((*user_id as u64).into())),
    )
    .await?;
    let user_nicknames_or_names = join_all(users.iter().map(|u| u.nick_in(ctx, message_guild_id)))
        .await
        .iter_mut()
        .zip(users)
        .map(|(n, u)| n.take().unwrap_or(u.name))
        .collect::<Vec<String>>();

    let mut fields: Vec<(String, String, bool)> = Vec::new();

    for (counter, (row, username)) in user_xp_lvl
        .iter()
        .zip(user_nicknames_or_names.iter())
        .enumerate()
    {
        let (level, xp) = (row.level, row.xp);
        let xp_to_level_up = calculate_xp_to_level_up(level);

        fields.push((
            format!("#{} >> {}", counter + 1, username),
            format!(
                "Lvl: {} | XP: {}/{} ({:.2}%)",
                level,
                xp,
                xp_to_level_up,
                ((xp as f64) / (xp_to_level_up as f64)) * 100.
            ),
            false,
        ));
    }

    let response = format!(
        "Guild: {}\n\nTop 9 Users",
        ctx.guild().ok_or("No guild found")?.name
    );
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
