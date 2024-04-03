use super::*;
use crate::commands::cmd_utils::{get_bot_avatar, get_bot_user};
use crate::data::command_data::{Context, Error};
use crate::enums::command_enums::EmbedType;
use ::serenity::all::Mentionable;

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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::TieUp).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
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
            target_replied_user.mention()
        ))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *ties up* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Pat).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
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

    let response: String = format!(
        "**{}** *pats* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Hug).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
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

    let response: String = format!(
        "**{}** *hugs* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content("Aww~ I won't kiss you! Ahahahah!")
                .embed(
                    serenity::CreateEmbed::new()
                        .color((255, 0, 0))
                        .image(
                            cmd_utils::get_embed_from_type(&EmbedType::Slap)
                                .await?
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
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kiss).await?;

    let response: String = format!(
        "**{}** *kisses* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Slap).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
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
        ctx.reply(format!(
            "Did you like it? {}",
            target_replied_user.mention()
        ))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *slaps* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Punch).await?;
    if target_replied_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content("I won't punch you! *pouts*"))
            .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *punches* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Bonk).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
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

    let response: String = format!(
        "**{}** *bonks* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Nom).await?;
    let bot_user = get_bot_user(ctx).await;
    if target_replied_user == ctx.author() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("{} noms themselves...?", target_replied_user.name))
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

    let response: String = format!(
        "**{}** *noms* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kill).await?;
    if target_replied_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content("No."))
            .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *kills* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Kick).await?;
    if target_replied_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content(format!(
            "{}, why would you kick yourself...? Weirdo...",
            target_replied_user.mention()
        )))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *kicks* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Bury).await?;
    if target_replied_user == ctx.author() {
        ctx.send(poise::CreateReply::default().content(format!(
            "{} Just use the `!selfbury` or `/selfbury` command bruh...",
            target_replied_user.mention()
        )))
        .await?;
        return Ok(());
    }

    let response: String = format!(
        "**{}** *buries* **{}**",
        ctx.author().name,
        target_replied_user.name
    );

    let ping_on_shash_command: Option<poise::serenity_prelude::Mention> = match ctx {
        poise::Context::Prefix(_) => None,
        poise::Context::Application(_) => Some(target_replied_user.mention()),
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
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::SelfBury).await?;
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
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Peek).await?;
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
    let target_replied_user = user.as_ref().unwrap_or(ctx.get_replied_msg_author());
    let response: String = format!("**{}**'s avatar:", target_replied_user.name);
    let user_avatar_as_embed: String = target_replied_user.face().replace(".webp", ".png");

    let embed = serenity::CreateEmbed::new()
        .title(response)
        .color((255, 0, 0))
        .image(user_avatar_as_embed);
    let full_respone = poise::CreateReply::default().embed(embed);
    ctx.send(full_respone).await?;

    Ok(())
}

/// Get a Ryan Gosling drive GIF.
#[poise::command(slash_command, prefix_command, rename = "drive")]
pub async fn drive(ctx: Context<'_>) -> Result<(), Error> {
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::RyanGoslingDrive).await?;

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
    let embed_item: &str = cmd_utils::get_embed_from_type(&EmbedType::Chair).await?;
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

/// Just try it.
#[poise::command(slash_command, prefix_command, rename = "boom")]
pub async fn boom(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = ctx.http().get_current_user().await?;
    let bot_avatar = bot_user.face().replace(".webp", ".png");

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .image("https://cdn.discordapp.com/attachments/1129364410566193192/1223359511356641321/ea022b6f5e25129f8c865b6b2d8e2f33.jpg?ex=66199154&is=66071c54&hm=3fc8357942f1ea01c76b2c249c6db654ef6572d00a5e7d65af4de3266d39ae6b&")
                .color((255, 0, 0))
                .footer(serenity::CreateEmbedFooter::new(bot_user.tag()).icon_url(bot_avatar)),
        ),
    )
    .await?;

    Ok(())
}
