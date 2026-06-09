use crate::prelude::*;
use serenity_discord_bot_derive::embed_commands;

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command_utility",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn get_name(ctx: &Context<'_>, guild_id: Option<GuildId>, u: &serenity::User) -> String {
    let base_case = || u.name.clone();
    match guild_id {
        Some(id) => u.nick_in(ctx, id).await.unwrap_or(base_case()),
        None => base_case(),
    }
}

#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command_utility",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
        user_1_id = %user_1.id,
        user_2_id = %user_2.id,
    )
)]
pub async fn user_interaction(
    ctx: &Context<'_>,
    guild_id: Option<GuildId>,
    user_1: &serenity::User,
    user_2: &serenity::User,
    f: fn(name_1: &str, name_2: &str) -> String,
) -> String {
    let [ref name_1, ref name_2] = join_all([
        get_name(ctx, guild_id, user_1),
        get_name(ctx, guild_id, user_2),
    ])
    .await[..] else {
        return String::from("");
    };

    f(name_1, name_2)
}

// The embed (GIF) commands, generated from one spec per command.
//
// `interaction` commands take an optional target user (falling back to the
// replied-to message's author) and handle self-targeting via `on_self`; `solo`
// commands just post the embed. Expressions in the spec run inside the command
// body, with `ctx` and (where a target exists) `target` in scope.
embed_commands! {
    // #region User interaction commands

    /// Tie someone up (HUH?)
    tieup => interaction {
        embed: TieUp,
        verb: "ties up",
        on_self: reply_embed("Y'know what? Sure, I'll tie you up!"),
        self_followup: format!(
            "Did you like it {}? You filthy degenerate~",
            target.mention()
        ),
    },

    /// Pat someone
    pat => interaction {
        embed: Pat,
        verb: "pats",
        on_self: reply_embed("Aww~ I'll pat you!"),
    },

    /// Hug someone
    hug => interaction {
        embed: Hug,
        verb: "hugs",
        on_self: reply_embed("Aww~ I'll hug you!"),
    },

    /// Kiss someone
    kiss => interaction {
        embed: Kiss,
        verb: "kisses",
        on_self: reply_embed_as(Slap, "Aww~ I won't kiss you! Ahahahah!"),
    },

    /// Slap someone
    slap => interaction {
        embed: Slap,
        verb: "slaps",
        on_self: reply_embed("Why do you want to get slapped??"),
        self_followup: format!("Did you like it? {}", target.mention()),
    },

    /// Punch someone
    punch => interaction {
        embed: Punch,
        verb: "punches",
        on_self: reply_text("I won't punch you! *pouts*"),
    },

    /// Bonk someone who's horknee
    bonk => interaction {
        embed: Bonk,
        verb: "bonks",
        on_self: reply_embed("バカ！"),
    },

    /// Nom someone
    nom => interaction {
        embed: Nom,
        verb: "noms",
        on_self: reply_embed(format!("{} noms themselves...?", target.name)),
    },

    /// Kill someone (Sadge)
    kill => interaction {
        embed: Kill,
        verb: "kills",
        on_self: reply_text("No."),
    },

    /// Kick someone
    kick => interaction {
        embed: Kick,
        verb: "kicks",
        on_self: reply_text(format!(
            "{}, why would you kick yourself...? Weirdo...",
            target.mention()
        )),
    },

    /// Bury someone
    bury => interaction {
        embed: Bury,
        verb: "buries",
        require_target: format!(
            "{} Just use the `!selfbury` or `/selfbury` command bruh...",
            ctx.author().mention()
        ),
    },

    /// Bury yourself (perhaps to help Hu Tao's busines idk...)
    selfbury => solo {
        embed: SelfBury,
        title: format!("**{}** *buries themselves*", ctx.author().name),
    },

    /// Send a peek GIF in the chat (you lurker)
    peek => solo {
        embed: Peek,
        title: format!("{} is lurking . . .", ctx.author().name),
    },

    // #endregion

    /// Get the avatar for someone.
    avatar => solo {
        target,
        image: target.face().replace(".webp", ".png"),
        title: format!("**{}**'s avatar:", target.name),
        footer: false,
    },

    /// Get a Ryan Gosling drive GIF.
    drive => solo {
        embed: RyanGoslingDrive,
    },

    /// Get a motivation chair GIF
    chair => solo {
        embed: Chair,
        title: "You need some motivation!",
    },

    /// Just try it.
    boom => solo {
        image: Assets::HuBoom.to_string(),
    },
}

pub mod quote;
pub use quote::quote;

pub mod uptime;
pub use uptime::uptime;
