use crate::commands::cmd_utils::get_replied_user;
use crate::data::bot_data::{DATABASE_COLUMNS, DATABASE_FILENAME};
use crate::data::command_data::{Context, Error};
use crate::data::database_interactions::{
    connect_to_db, fetch_top_nine_levels_in_guild, fetch_user_level_and_rank,
};
use crate::enums::schemas::DatabaseSchema::*;
use ::serenity::futures::future::try_join_all;
use poise::serenity_prelude as serenity;
use rayon::prelude::*;
use sqlx::Row;
use std::sync::Arc;

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
    let bot_avatar = Arc::clone(&ctx.data().bot_avatar).to_string();

    let thumbnail = ctx
        .guild()
        .and_then(|guild| guild.icon_url())
        .unwrap_or_else(|| bot_avatar.to_owned());

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title(response)
                .fields(fields)
                .thumbnail(thumbnail)
                .color((255, 0, 0))
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;

    Ok(())
}
