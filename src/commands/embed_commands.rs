use ::serenity::all::Mentionable;
use ::serenity::futures::future::try_join_all;

use super::*;
use crate::commands::cmd_utils::{get_bot_avatar, get_bot_user};
use crate::data::bot_data::{DATABASE_COLUMNS, DATABASE_FILENAME};
use crate::data::command_data::{Context, Error};
use crate::data::database_interactions::{
    connect_to_db, fetch_top_nine_levels_in_guild, fetch_user_level_and_rank,
};
use crate::enums::command_enums::EmbedType;
use crate::enums::schemas::DatabaseSchema::*;
use sqlx::Row;

// This is where the poise framework shines since with it you can make
// a slash and a prefix command work in one function.
//
// Docs for reference:
// https://docs.rs/poise/latest/poise/reply/fn.send_reply.html

// #region User interaction commands

/// Tie someone up (HUH?)
#[poise::command(prefix_command, slash_command, rename = "tieup")]
pub async fn tieup(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::TieUp).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Y'know what? Sure, I'll tie you up!")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(embed_item.to_string())
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        ctx.reply(format!(
            "Did you like it {}? You filthy degenerate~", // true...
            target_user.mention()
        ))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *ties up* **{}**",
        ctx.author().name,
        target_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Pat someone
#[poise::command(prefix_command, slash_command, rename = "pat")]
pub async fn pat(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Pat).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Aww~ I'll pat you!")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(embed_item.to_string())
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *pats* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Hug someone
#[poise::command(prefix_command, slash_command, rename = "hug")]
pub async fn hug(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Hug).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Aww~ I'll hug you!")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(embed_item.to_string())
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *hugs* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Kiss someone
#[poise::command(prefix_command, slash_command, rename = "kiss")]
pub async fn kiss(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Aww~ I won't kiss you! Ahahahah!")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(
                            cmd_utils::get_embed_from_type(&EmbedType::Slap)
                                .await
                                .to_string(),
                        )
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        return Ok(());
    }
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kiss).await;

    let response: String = format!(
        "**{}** *kisses* **{}**",
        ctx.author().name,
        target_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Slap someone
#[poise::command(prefix_command, slash_command, rename = "slap")]
pub async fn slap(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Slap).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Why do you want to get slapped??")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(embed_item.to_string())
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        ctx.reply(format!("Did you like it? {}", target_user.mention()))
            .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *slaps* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Punch someone
#[poise::command(prefix_command, slash_command, rename = "punch")]
pub async fn punch(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Punch).await;
    if &target_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content("I won't punch you! *pouts*"))
            .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *punches* **{}**",
        ctx.author().name,
        target_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Bonk someone who's horknee
#[poise::command(prefix_command, slash_command, rename = "bonk")]
pub async fn bonk(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Bonk).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default().content("バカ！").embed(
                serenity::CreateEmbed::new()
                    .color((255, 0, 0))
                    .image(embed_item.to_string())
                    .footer(
                        serenity::CreateEmbedFooter::new(bot_user.tag())
                            .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                    ),
            ),
        )
        .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *bonks* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Nom someone
#[poise::command(prefix_command, slash_command, rename = "nom")]
pub async fn nom(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Nom).await;
    let bot_user = get_bot_user(ctx).await;
    if &target_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("{} noms themselves...?", target_user.name))
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(embed_item.to_string())
                        .footer(
                            serenity::CreateEmbedFooter::new(bot_user.tag())
                                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
                        ),
                ),
        )
        .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *noms* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Kill someone (Sadge)
#[poise::command(prefix_command, slash_command, rename = "kill")]
pub async fn kill(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kill).await;
    if &target_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content("No."))
            .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *kills* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Kick someone
#[poise::command(prefix_command, slash_command, rename = "kick")]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kick).await;
    if &target_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content(format!(
            "{}, why would you kick yourself...? Weirdo...",
            target_user.mention()
        )))
        .await?;
        return Ok(());
    }

    let response: String = format!("**{}** *kicks* **{}**", ctx.author().name, target_user.name);

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };

    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Bury someone
#[poise::command(prefix_command, slash_command, rename = "bury")]
pub async fn bury(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = cmd_utils::get_user(ctx, user).await;
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Bury).await;
    if &target_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content(format!(
            "{} Just use the `!selfbury` or `/selfbury` command bruh...",
            target_user.mention()
        )))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *buries* **{}**",
        ctx.author().name,
        target_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_user.mention()),
    };
    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default()
        .content(match ping_on_shash_command {
            Some(ping) => format!("{}", ping),
            None => "".to_owned(),
        })
        .embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Bury yourself (perhaps to help Hu Tao's busines idk...)
#[poise::command(prefix_command, slash_command, rename = "selfbury")]
pub async fn selfbury(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::SelfBury).await;
    let response: String = format!("**{}** *buries themselves*", ctx.author().name,);
    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Send a peek GIF in the chat (you lurker)
#[poise::command(prefix_command, slash_command, rename = "peek")]
pub async fn peek(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Peek).await;
    let response: String = format!("{} is lurking . . .", ctx.author().name,);
    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(embed_item.to_string())
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );

    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

// #endregion

/// Get the avatar for someone.
#[poise::command(slash_command, prefix_command, rename = "avatar")]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user: poise::serenity_prelude::User = cmd_utils::get_user(ctx, user).await;
    let response: String = format!("**{}**'s avatar:", target_user.name);
    let user_avatar_as_embed: String = target_user.face().replace(".webp", ".png");

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(user_avatar_as_embed);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

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
    let selected_user = cmd_utils::get_user(ctx, user).await;
    let db = connect_to_db(DATABASE_FILENAME.to_string()).await;
    let level_xp_and_rank_row_option = match db.await {
        Ok(pool) => {
            println!("Connected to the database: {pool:?}");
            fetch_user_level_and_rank(&pool, &selected_user, message_guild_id).await?
        }
        Err(_) => {
            ctx.reply(format!(
                "Please wait for {} to chat more then try again later...",
                selected_user.name
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
            selected_user.name
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

    let avatar = selected_user.face().replace(".webp", ".png");
    let username = selected_user.name;
    let response = format!(
        "User stats for: **{}**\n\nRank: {}",
        &username, level_xp_and_rank_row.0
    );
    let bot_user = ctx
        .http()
        .get_user(ctx.framework().bot_id)
        .await
        .expect("Retrieving the bot user shouldn't fail.");
    let bot_avatar = bot_user.face().replace(".webp", ".png");
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
    let user_ids: Vec<u64> = level_and_xp_rows
        .iter()
        .map(|row| row.get::<i64, &str>(DATABASE_COLUMNS[&UserId]) as u64)
        .collect();
    let users = try_join_all(
        user_ids
            .into_iter()
            .map(|user_id| ctx.http().get_user(user_id.into())),
    );

    let mut fields: Vec<(String, String, bool)> = Vec::new();

    for (counter, (row, user)) in level_and_xp_rows
        .iter()
        .zip(users.await?.into_iter())
        .enumerate()
        .take(9)
    {
        let (level, xp) = (
            row.get::<i32, &str>(DATABASE_COLUMNS[&Level]),
            row.get::<i32, &str>(DATABASE_COLUMNS[&ExperiencePoints]),
        );

        let name = if user.id == 1 {
            "Unknown user.".into()
        } else {
            user.name
        };

        fields.push((
            format!("#{} >> {}", counter + 1, name,),
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
    let bot_user = ctx.http().get_current_user().await?;
    let bot_avatar = bot_user.face().replace(".webp", ".png");

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

/// Get a Ryan Gosling drive GIF.
#[poise::command(slash_command, prefix_command, rename = "drive")]
pub async fn drive(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::RyanGoslingDrive).await;

    let embed = serenity::CreateEmbed::new()
        // .title()
        .color((255, 0, 0))
        .image(embed_item);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Get a motivation chair GIF
#[poise::command(slash_command, prefix_command, rename = "chair")]
pub async fn chair(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Chair).await;
    let bot_user = get_bot_user(ctx).await;

    let embed = serenity::CreateEmbed::new()
        .title("You need some motivation!")
        .color((255, 0, 0))
        .image(embed_item)
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(get_bot_avatar(ctx, Some(bot_user)).await),
        );
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}
