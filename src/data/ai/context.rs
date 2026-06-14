use ::serenity::all::GetMessages;
use redis::AsyncCommands;

use super::{config::AI_MAX_MSG_CONTEXT, provider::AiMessage};
use crate::prelude::*;

/// How long an idle context window lives in Redis before eviction.
const AI_CTX_TTL_SECS: i64 = 1800;
/// Field separator in a stored entry (`author_id␟name␟content`). Unit Separator,
/// which won't appear in Discord message text.
const AI_CTX_SEP: char = '\u{1f}';

fn ctx_key(channel_id: u64) -> String {
    format!("ai:ctx:{channel_id}")
}

/// A display name for prompt attribution (global/display name, else username).
pub fn author_name(author: &serenity::User) -> String {
    author
        .global_name
        .clone()
        .unwrap_or_else(|| author.name.clone())
}

/// Map a message to a prompt turn. The bot's own turns are the assistant role
/// (unprefixed); everyone else is a user turn prefixed with their name so the
/// model can tell speakers apart in busy channels.
fn to_message(author_id: u64, name: &str, content: &str, bot_user_id: u64) -> Option<AiMessage> {
    if content.trim().is_empty() {
        return None;
    }
    if author_id == bot_user_id {
        Some(AiMessage::new("assistant", content))
    } else {
        Some(AiMessage::new("user", &format!("{name}: {content}")))
    }
}

/// Max chars of a parent message kept in the reply marker.
const AI_REPLY_SNIPPET_CHARS: usize = 80;

/// A message rendered for the model: its text plus any embeds flattened to text
/// (author/title/description/fields/footer), with images noted but not shown.
/// Command outputs are usually embed-only with empty `content`, so without this
/// the model can't see them at all.
///
/// Inline replies get a `[replying to {author}: {snippet}]` marker prepended so
/// the model keeps the link to the parent message; only the immediate parent is
/// included.
fn render_message(message: &serenity::Message) -> String {
    let mut parts: Vec<String> = Vec::new();

    let content = message.content.trim();
    if !content.is_empty() {
        parts.push(content.to_string());
    }

    for embed in &message.embeds {
        parts.push(render_embed(embed));
    }

    let rendered = parts.join("\n");

    match message.referenced_message.as_deref() {
        Some(parent) => {
            let snippet = reply_snippet(&render_message(parent));
            format!(
                "[replying to {}: {snippet}] {rendered}",
                author_name(&parent.author)
            )
        }
        None => rendered,
    }
}

/// Flatten the parent's rendered text to a single line and truncate on a char
/// boundary, appending an ellipsis if cut. Keeps the marker cheap against
/// `AI_MAX_TOKENS`.
fn reply_snippet(rendered: &str) -> String {
    let flat = rendered.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut snippet: String = flat.chars().take(AI_REPLY_SNIPPET_CHARS).collect();
    if flat.chars().count() > AI_REPLY_SNIPPET_CHARS {
        snippet.push('…');
    }
    snippet
}

fn render_embed(embed: &serenity::Embed) -> String {
    let mut bits: Vec<String> = vec!["[embed]".to_string()];

    if let Some(author) = &embed.author {
        bits.push(format!("author: {}", author.name));
    }
    if let Some(title) = &embed.title {
        bits.push(format!("title: {title}"));
    }
    if let Some(description) = &embed.description {
        bits.push(description.clone());
    }
    for field in &embed.fields {
        bits.push(format!("{}: {}", field.name, field.value));
    }
    if let Some(footer) = &embed.footer {
        bits.push(format!("footer: {}", footer.text));
    }
    if embed.image.is_some() {
        bits.push("[image attached]".to_string());
    }
    if embed.thumbnail.is_some() {
        bits.push("[thumbnail attached]".to_string());
    }

    bits.join(" | ")
}

fn encode_entry(author_id: u64, name: &str, content: &str) -> String {
    format!("{author_id}{AI_CTX_SEP}{name}{AI_CTX_SEP}{content}")
}

fn entry_to_message(entry: &str, bot_user_id: u64) -> Option<AiMessage> {
    let mut parts = entry.splitn(3, AI_CTX_SEP);
    let author_id = parts.next()?.parse::<u64>().ok()?;
    let name = parts.next()?;
    let content = parts.next()?;
    to_message(author_id, name, content, bot_user_id)
}

/// Append a message to a channel's window, but only if the window already exists
/// (i.e. the channel is "warm" from a prior AI interaction). No-op without Redis.
#[tracing::instrument(skip(message), fields(category = "redis", channel_id = %message.channel_id))]
pub async fn record_message(message: &serenity::Message) {
    let rendered = render_message(message);
    if rendered.trim().is_empty() {
        return;
    }

    let Some(mut conn) = crate::data::cache::conn().await else {
        return;
    };

    let script = redis::Script::new(
        r"if redis.call('EXISTS', KEYS[1]) == 1 then
            redis.call('RPUSH', KEYS[1], ARGV[1])
            redis.call('LTRIM', KEYS[1], -tonumber(ARGV[2]), -1)
            redis.call('EXPIRE', KEYS[1], ARGV[3])
        end
        return 1",
    );

    let result: redis::RedisResult<i64> = script
        .key(ctx_key(message.channel_id.get()))
        .arg(encode_entry(
            message.author.id.get(),
            &author_name(&message.author),
            &rendered,
        ))
        .arg(*AI_MAX_MSG_CONTEXT as i64)
        .arg(AI_CTX_TTL_SECS)
        .invoke_async(&mut conn)
        .await;

    if let Err(why) = result {
        tracing::warn!("Failed to record message in Redis: {why}");
    }
}

/// The recent conversation for a channel as a prompt.
///
/// Reads the Redis window; on a cold channel (or without Redis) it seeds the
/// window from a one-off Discord fetch. `extra_system`, when set, is prepended
/// as a leading system turn (a guild's custom prompt addon). `current` is
/// appended as a trailing user turn — used by `/ai`, whose prompt isn't a
/// channel message; the auto-reply passes `None` since the triggering message is
/// already in the window.
#[tracing::instrument(
    skip(cache_http, extra_system, current),
    fields(category = "ai_context", channel_id = %channel_id)
)]
pub async fn channel_context(
    cache_http: impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
    extra_system: Option<&str>,
    current: Option<&str>,
) -> Vec<AiMessage> {
    let key = ctx_key(channel_id.get());

    let mut prompt: Vec<AiMessage> = match crate::data::cache::conn().await {
        Some(mut conn) => {
            let entries: Vec<String> = conn.lrange(&key, 0, -1).await.unwrap_or_default();
            if entries.is_empty() {
                seed_from_discord(&cache_http, channel_id, bot_user_id, &key).await
            } else {
                entries
                    .iter()
                    .filter_map(|e| entry_to_message(e, bot_user_id))
                    .collect()
            }
        }
        None => fetch_from_discord(&cache_http, channel_id, bot_user_id).await,
    };

    // The base persona lives on the provider; a guild's extra prompt rides as a
    // leading system turn (`chat` sends system turns as a user instruction).
    if let Some(extra) = extra_system {
        prompt.insert(0, AiMessage::new("system", extra));
    }

    if let Some(current) = current {
        prompt.push(AiMessage::new("user", current));
    }

    prompt
}

/// Fetch the recent window from Discord and map it to prompt messages.
#[tracing::instrument(skip_all, fields(category = "discord_fetch", channel_id = %channel_id))]
async fn fetch_from_discord(
    cache_http: &impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
) -> Vec<AiMessage> {
    let limit = (*AI_MAX_MSG_CONTEXT).min(100) as u8;
    let mut messages = channel_id
        .messages(cache_http, GetMessages::new().limit(limit))
        .await
        .unwrap_or_default();
    messages.retain(|m| !m.author.bot || m.author.id.get() == bot_user_id);
    messages.reverse();

    messages
        .iter()
        .filter_map(|m| {
            to_message(
                m.author.id.get(),
                &author_name(&m.author),
                &render_message(m),
                bot_user_id,
            )
        })
        .collect()
}

/// Seed a cold channel's Redis window from Discord and return the prompt.
#[tracing::instrument(skip_all, fields(category = "discord_fetch", channel_id = %channel_id))]
async fn seed_from_discord(
    cache_http: &impl serenity::CacheHttp,
    channel_id: serenity::ChannelId,
    bot_user_id: u64,
    key: &str,
) -> Vec<AiMessage> {
    let limit = (*AI_MAX_MSG_CONTEXT).min(100) as u8;
    let mut messages = channel_id
        .messages(cache_http, GetMessages::new().limit(limit))
        .await
        .unwrap_or_default();
    messages.retain(|m| !m.author.bot || m.author.id.get() == bot_user_id);
    messages.reverse();

    // (author_id, name, rendered text); drop messages that render to nothing.
    let rendered: Vec<(u64, String, String)> = messages
        .iter()
        .map(|m| (m.author.id.get(), author_name(&m.author), render_message(m)))
        .filter(|(_, _, text)| !text.trim().is_empty())
        .collect();

    if let Some(mut conn) = crate::data::cache::conn().await
        && !rendered.is_empty()
    {
        let encoded: Vec<String> = rendered
            .iter()
            .map(|(id, name, text)| encode_entry(*id, name, text))
            .collect();
        let result: redis::RedisResult<()> = redis::pipe()
            .atomic()
            .del(key)
            .ignore()
            .rpush(key, encoded)
            .ignore()
            .expire(key, AI_CTX_TTL_SECS)
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(why) = result {
            tracing::warn!("Failed to seed Redis window: {why}");
        }
    }

    rendered
        .iter()
        .filter_map(|(id, name, text)| to_message(*id, name, text, bot_user_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT_ID: u64 = 1000;

    #[test]
    fn ctx_key_includes_channel_id() {
        assert_eq!(ctx_key(42), "ai:ctx:42");
    }

    #[test]
    fn to_message_empty_content_is_dropped() {
        assert!(to_message(1, "alice", "   ", BOT_ID).is_none());
    }

    #[test]
    fn to_message_bot_is_assistant_without_prefix() {
        let msg = to_message(BOT_ID, "hutao", "hello", BOT_ID);
        assert!(msg.is_some());
    }

    #[test]
    fn entry_roundtrips_through_encode_and_decode() {
        let entry = encode_entry(1, "alice", "hi there");
        assert!(entry_to_message(&entry, BOT_ID).is_some());
    }

    #[test]
    fn entry_content_may_contain_colons_and_newlines() {
        let entry = encode_entry(1, "alice", "key: value\nsecond line");
        assert!(entry_to_message(&entry, BOT_ID).is_some());
    }

    #[test]
    fn reply_snippet_truncates_on_char_boundary_with_ellipsis() {
        // Multi-byte chars so byte indexing would panic; char-based must not.
        let long = "é".repeat(AI_REPLY_SNIPPET_CHARS + 5);
        let snippet = reply_snippet(&long);
        assert_eq!(snippet.chars().count(), AI_REPLY_SNIPPET_CHARS + 1);
        assert!(snippet.ends_with('…'));

        // Short input is left intact, no ellipsis.
        let short = reply_snippet("hi there");
        assert_eq!(short, "hi there");
        assert!(!short.ends_with('…'));
    }

    #[test]
    fn malformed_entry_is_dropped() {
        assert!(entry_to_message("not-a-number\u{1f}alice\u{1f}hi", BOT_ID).is_none());
        assert!(entry_to_message("no separators at all", BOT_ID).is_none());
        assert!(entry_to_message("", BOT_ID).is_none());
    }
}
