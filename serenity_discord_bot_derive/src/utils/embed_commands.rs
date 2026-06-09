//! Parsing and expansion for the `embed_commands!` macro.
//!
//! The macro generates the embed (GIF) commands, which all share the same
//! shape: pick a random embed for the command's `EmbedType`, build the red
//! embed with the bot footer, and send it. Two kinds exist:
//!
//! - `interaction`: takes an optional target user, titles the embed with
//!   "**author** *verb* **target**", and handles the self-target case.
//! - `solo`: no interaction line; just the embed, with an optional title,
//!   optional target user, and an optional non-random image source.
//!
//! Expressions in the spec (self replies, titles, images) are expanded
//! inside the generated command body, so they can use `ctx` and — where a
//! target exists — `target` (`&serenity::User`).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Expr, Ident, LitBool, LitStr, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
};

pub struct EmbedCommands {
    commands: Vec<CommandSpec>,
}

struct CommandSpec {
    docs: Vec<Attribute>,
    name: Ident,
    kind: CommandKind,
}

enum CommandKind {
    Interaction(InteractionSpec),
    Solo(SoloSpec),
}

struct InteractionSpec {
    /// `EmbedType` variant the random GIF is picked from.
    embed: Ident,
    /// Verb in the "**author** *verb* **target**" title.
    verb: LitStr,
    /// What to do when the author targets themselves.
    on_self: Option<OnSelf>,
    /// Optional plain reply sent after the self-target response.
    self_followup: Option<Expr>,
    /// Reply with this message instead of resolving a target from the
    /// replied-to message (mutually exclusive with `on_self`).
    require_target: Option<Expr>,
}

enum OnSelf {
    /// Send the command's embed with this content.
    ReplyEmbed(Expr),
    /// Send an embed from a different `EmbedType` with this content.
    ReplyEmbedAs(Ident, Expr),
    /// Send a plain text reply, no embed.
    ReplyText(Expr),
}

struct SoloSpec {
    /// Whether the command takes an optional target user.
    target: bool,
    /// `EmbedType` variant for a random GIF (mutually exclusive with `image`).
    embed: Option<Ident>,
    /// Explicit image expression (mutually exclusive with `embed`).
    image: Option<Expr>,
    title: Option<Expr>,
    /// Whether to attach the bot footer (defaults to true).
    footer: bool,
}

impl Parse for EmbedCommands {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut commands = Vec::new();
        while !input.is_empty() {
            commands.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { commands })
    }
}

impl Parse for CommandSpec {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let docs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let kind_ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let kind = match kind_ident.to_string().as_str() {
            "interaction" => CommandKind::Interaction(parse_interaction(&content, &name)?),
            "solo" => CommandKind::Solo(parse_solo(&content, &name)?),
            other => {
                return Err(syn::Error::new(
                    kind_ident.span(),
                    format!("expected `interaction` or `solo`, found `{other}`"),
                ));
            }
        };
        Ok(Self { docs, name, kind })
    }
}

fn parse_interaction(input: ParseStream<'_>, name: &Ident) -> syn::Result<InteractionSpec> {
    let mut embed = None;
    let mut verb = None;
    let mut on_self = None;
    let mut self_followup = None;
    let mut require_target = None;

    while !input.is_empty() {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        match key.to_string().as_str() {
            "embed" => embed = Some(input.parse()?),
            "verb" => verb = Some(input.parse()?),
            "on_self" => on_self = Some(parse_on_self(input)?),
            "self_followup" => self_followup = Some(input.parse()?),
            "require_target" => require_target = Some(input.parse()?),
            other => {
                return Err(syn::Error::new(
                    key.span(),
                    format!("unknown interaction field `{other}`"),
                ));
            }
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    let err = |msg: &str| syn::Error::new(name.span(), format!("`{name}`: {msg}"));
    let embed = embed.ok_or_else(|| err("missing `embed`"))?;
    let verb = verb.ok_or_else(|| err("missing `verb`"))?;
    if on_self.is_some() == require_target.is_some() {
        return Err(err("set exactly one of `on_self` or `require_target`"));
    }
    if self_followup.is_some() && on_self.is_none() {
        return Err(err("`self_followup` needs `on_self`"));
    }

    Ok(InteractionSpec {
        embed,
        verb,
        on_self,
        self_followup,
        require_target,
    })
}

fn parse_on_self(input: ParseStream<'_>) -> syn::Result<OnSelf> {
    let which: Ident = input.parse()?;
    let args;
    parenthesized!(args in input);
    match which.to_string().as_str() {
        "reply_embed" => Ok(OnSelf::ReplyEmbed(args.parse()?)),
        "reply_embed_as" => {
            let variant: Ident = args.parse()?;
            args.parse::<Token![,]>()?;
            Ok(OnSelf::ReplyEmbedAs(variant, args.parse()?))
        }
        "reply_text" => Ok(OnSelf::ReplyText(args.parse()?)),
        other => Err(syn::Error::new(
            which.span(),
            format!("expected `reply_embed`, `reply_embed_as`, or `reply_text`, found `{other}`"),
        )),
    }
}

fn parse_solo(input: ParseStream<'_>, name: &Ident) -> syn::Result<SoloSpec> {
    let mut spec = SoloSpec {
        target: false,
        embed: None,
        image: None,
        title: None,
        footer: true,
    };

    while !input.is_empty() {
        let key: Ident = input.parse()?;
        if key == "target" {
            spec.target = true;
        } else {
            input.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "embed" => spec.embed = Some(input.parse()?),
                "image" => spec.image = Some(input.parse()?),
                "title" => spec.title = Some(input.parse()?),
                "footer" => spec.footer = input.parse::<LitBool>()?.value,
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown solo field `{other}`"),
                    ));
                }
            }
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    if spec.embed.is_some() == spec.image.is_some() {
        return Err(syn::Error::new(
            name.span(),
            format!("`{name}`: set exactly one of `embed` or `image`"),
        ));
    }

    Ok(spec)
}

pub fn expand(cmds: &EmbedCommands) -> TokenStream {
    let items = cmds.commands.iter().map(|cmd| match &cmd.kind {
        CommandKind::Interaction(spec) => expand_interaction(cmd, spec),
        CommandKind::Solo(spec) => expand_solo(cmd, spec),
    });
    quote! { #(#items)* }
}

/// The shared bot footer call; expects `bot_user` in scope.
fn footer() -> TokenStream {
    quote! {
        .footer(
            serenity::CreateEmbedFooter::new(bot_user.tag())
                .icon_url(Arc::clone(&ctx.data().bot_avatar).to_string()),
        )
    }
}

fn expand_interaction(cmd: &CommandSpec, spec: &InteractionSpec) -> TokenStream {
    let docs = &cmd.docs;
    let name = &cmd.name;
    let embed = &spec.embed;
    let footer = footer();
    let verb_fmt = LitStr::new(
        &format!("**{{}}** *{}* **{{}}**", spec.verb.value()),
        spec.verb.span(),
    );

    let (target_let, self_case) = if let Some(msg) = &spec.require_target {
        let target_let = quote! {
            let Some(target) = user.as_ref() else {
                ctx.send(poise::CreateReply::default().content(#msg)).await?;
                return Ok(());
            };
        };
        (target_let, quote! {})
    } else {
        // Validated in parsing: `on_self` is set when `require_target` isn't.
        let reply = match &spec.on_self {
            Some(OnSelf::ReplyEmbed(content)) => quote! {
                ctx.send(
                    poise::CreateReply::default().content(#content).embed(
                        serenity::CreateEmbed::new()
                            .color((255, 0, 0))
                            .image(embed_item.to_string())
                            #footer,
                    ),
                )
                .await?;
            },
            Some(OnSelf::ReplyEmbedAs(variant, content)) => quote! {
                ctx.send(
                    poise::CreateReply::default().content(#content).embed(
                        serenity::CreateEmbed::new()
                            .color((255, 0, 0))
                            .image(
                                cmd_utils::get_rand_embed_from_type(&EmbedType::#variant)?
                                    .to_string(),
                            )
                            #footer,
                    ),
                )
                .await?;
            },
            Some(OnSelf::ReplyText(content)) => quote! {
                ctx.send(poise::CreateReply::default().content(#content)).await?;
            },
            None => unreachable!(),
        };
        let followup = spec
            .self_followup
            .as_ref()
            .map(|msg| quote! { ctx.reply(#msg).await?; });
        let target_let = quote! {
            let target = user.as_ref().unwrap_or(get_replied_user(ctx).await);
        };
        let self_case = quote! {
            if same_user(target, ctx.author()) {
                #reply
                #followup
                return Ok(());
            }
        };
        (target_let, self_case)
    };

    quote! {
        #(#docs)*
        #[poise::command(discard_spare_arguments, prefix_command, slash_command)]
        #[tracing::instrument(
            skip(ctx),
            fields(
                category = "discord_command",
                command.name = %ctx.command().name,
                author = %ctx.author().id,
                target_user = %user.as_ref().map(|u| u.id.get()).unwrap_or(0),
                guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
            )
        )]
        pub async fn #name(
            ctx: Context<'_>,
            #[description = "Selected user"] user: Option<serenity::User>,
        ) -> Result<(), Error> {
            let embed_item: &str = cmd_utils::get_rand_embed_from_type(&EmbedType::#embed)?;
            let bot_user = Arc::clone(&ctx.data().bot_user);
            #target_let
            #self_case

            let response: String = user_interaction(
                &ctx,
                ctx.guild_id(),
                ctx.author(),
                target,
                |u1, u2| format!(#verb_fmt, u1, u2),
            )
            .await;

            let embed = serenity::CreateEmbed::new()
                .title(response)
                .color((255, 0, 0))
                .image(embed_item.to_string())
                #footer;

            let full_response = make_full_response(&ctx, target, Some(embed)).await;
            ctx.send(full_response).await?;

            Ok(())
        }
    }
}

fn expand_solo(cmd: &CommandSpec, spec: &SoloSpec) -> TokenStream {
    let docs = &cmd.docs;
    let name = &cmd.name;

    let target_param = spec.target.then(|| {
        quote! { , #[description = "Selected user"] user: Option<serenity::User> }
    });
    let target_let = spec.target.then(|| {
        quote! { let target = user.as_ref().unwrap_or(get_replied_user(ctx).await); }
    });
    let tracing_target = spec.target.then(|| {
        quote! { target_user = %user.as_ref().map(|u| u.id.get()).unwrap_or(0), }
    });

    let image_expr = match (&spec.embed, &spec.image) {
        (Some(variant), None) => quote! {
            cmd_utils::get_rand_embed_from_type(&EmbedType::#variant)?.to_string()
        },
        (None, Some(expr)) => quote! { #expr },
        // Validated in parsing: exactly one of `embed`/`image` is set.
        _ => unreachable!(),
    };
    let title_call = spec.title.as_ref().map(|title| quote! { .title(#title) });
    let (bot_user_let, footer_call) = if spec.footer {
        let footer = footer();
        (
            quote! { let bot_user = Arc::clone(&ctx.data().bot_user); },
            footer,
        )
    } else {
        (quote! {}, quote! {})
    };

    quote! {
        #(#docs)*
        #[poise::command(discard_spare_arguments, prefix_command, slash_command)]
        #[tracing::instrument(
            skip(ctx),
            fields(
                category = "discord_command",
                command.name = %ctx.command().name,
                author = %ctx.author().id,
                #tracing_target
                guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
            )
        )]
        pub async fn #name(ctx: Context<'_> #target_param) -> Result<(), Error> {
            #target_let
            #bot_user_let
            let embed = serenity::CreateEmbed::new()
                #title_call
                .color((255, 0, 0))
                .image(#image_expr)
                #footer_call;

            ctx.send(poise::CreateReply::default().embed(embed)).await?;

            Ok(())
        }
    }
}
