//! Write-through Redis cache over the `custom_reactions` table.
//!
//! Hot path: one SISMEMBER on `cr:guilds`, then HGETALL on `cr:meta:{guild_id}`.
//! Cold start: `fetch_all_live` populates every guild's hash, gated by `cr:seeded`.
//! No Redis: falls back to a direct DB query.

use crate::{enums::schemas::CustomReactionsTable, prelude::*};

#[cfg(feature = "redis")]
use crate::data::cache;

/// Maximum pattern length accepted at registration time.
pub const PATTERN_MAX_LEN: usize = 512;

/// Compiled-automaton size limit passed to `RegexBuilder`.
const REGEX_SIZE_LIMIT: usize = 1 << 20; // 1 MiB
const REGEX_DFA_SIZE_LIMIT: usize = 1 << 20; // 1 MiB

/// Discord caps autocomplete responses at 25 entries.
const MAX_AUTOCOMPLETE: usize = 25;

const CR_SEEDED_KEY: &str = "cr:seeded";
const CR_GUILDS_KEY: &str = "cr:guilds";

fn meta_key(guild_id: i64) -> String {
    format!("cr:meta:{guild_id}")
}

// ── Per-process compiled-regex cache ──────────────────────────────────────────

/// Compiled regexes keyed by reaction id, so the per-message hot path never
/// recompiles. Ids are globally unique (IDENTITY) and patterns are immutable
/// (removing then re-adding yields a new id), so an id is a stable key; entries
/// are evicted on remove.
static COMPILED: LazyLock<std::sync::RwLock<HashMap<i64, Arc<Regex>>>> =
    LazyLock::new(|| std::sync::RwLock::new(HashMap::new()));

/// Compiled regex for a stored reaction, compiling and caching on first use.
/// Returns `None` only if a stored pattern fails to compile, which shouldn't
/// happen since patterns are validated at register time.
fn compiled_regex(id: i64, pattern: &str, anywhere: bool) -> Option<Arc<Regex>> {
    if let Some(re) = COMPILED.read().ok()?.get(&id) {
        return Some(Arc::clone(re));
    }
    let re = Arc::new(compile_pattern(pattern, anywhere).ok()?);
    if let Ok(mut map) = COMPILED.write() {
        map.insert(id, Arc::clone(&re));
    }
    Some(re)
}

/// Drop a reaction's compiled regex from the cache (on remove).
fn evict_compiled(id: i64) {
    if let Ok(mut map) = COMPILED.write() {
        map.remove(&id);
    }
}

/// Validate and compile a pattern string from user input.
///
/// Returns the compiled `Regex` on success, or a human-readable error string
/// that includes the regex101 Rust-flavor link and the docs.rs syntax reference.
pub fn compile_pattern(pattern: &str, anywhere: bool) -> Result<Regex, String> {
    if pattern.len() > PATTERN_MAX_LEN {
        return Err(format!(
            "Pattern is {} chars; the limit is {PATTERN_MAX_LEN}. \
             See https://docs.rs/regex/latest/regex/#syntax for the Rust regex flavor.",
            pattern.len()
        ));
    }

    let source = if anywhere {
        pattern.to_string()
    } else {
        format!("^(?:{pattern})$")
    };

    let re = RegexBuilder::new(&source)
        .size_limit(REGEX_SIZE_LIMIT)
        .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
        .build()
        .map_err(|e| {
            format!(
                "Pattern failed to compile: {e}\n\
                 Tip: use the Rust regex flavor (no lookahead/backrefs) — \
                 https://regex101.com/?flavor=rust \
                 https://docs.rs/regex/latest/regex/#syntax"
            )
        })?;

    if re.is_match("") {
        return Err(
            "Pattern matches the empty string (e.g. `.*`, `a?`, blank). \
             Use a pattern that requires at least one character."
                .to_string(),
        );
    }

    Ok(re)
}

/// Reject share/page links that aren't images. A `tenor.com/view/...` or
/// `giphy.com/gifs/...` URL is an HTML page, not a media file — Discord renders
/// those natively only when a *user* types them, never in a bot embed's image
/// field, so they show up as a blank box. Direct media URLs (`media.tenor.com`,
/// `media.giphy.com`, Discord CDN, anything else) pass through unchanged.
pub fn validate_image_url(url: &str) -> Result<(), String> {
    let lower = url.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err("Provide an http(s) image URL, or attach the file directly.".to_string());
    }
    let after_scheme = lower
        .split_once("://")
        .map_or(lower.as_str(), |(_, rest)| rest);
    let (host, path) = after_scheme.split_once('/').unwrap_or((after_scheme, ""));
    let host = host.trim_start_matches("www.");
    // Drop any `:port` so `tenor.com:443/view/...` is still recognized.
    let host = host.split_once(':').map_or(host, |(h, _)| h);
    // Compare the first path segment, so `…/view` (no trailing slash) and
    // `…//view/x` (double slash) are caught, not just `…/view/x`.
    let first_seg = path
        .split(['/', '?', '#'])
        .find(|s| !s.is_empty())
        .unwrap_or("");

    if host == "tenor.com" && first_seg == "view" {
        return Err("That's a Tenor page link, not an image. Open the GIF, \
                    right-click it, copy the direct media URL (it ends in `.gif` \
                    on `media.tenor.com`), and use that."
            .to_string());
    }
    if host == "giphy.com" && matches!(first_seg, "gifs" | "clips" | "stickers") {
        return Err(
            "That's a Giphy page link, not an image. Open the GIF, copy \
                    the direct media URL (it ends in `.gif` on `media.giphy.com`), \
                    and use that."
                .to_string(),
        );
    }
    Ok(())
}

/// 80-char preview of a pattern for display in `list` and the `remove`
/// autocomplete, with a trailing `…` when truncated. Both surfaces share this so
/// the autocomplete value and the list always show the same text, and `remove`
/// can match the selected preview against the current row.
pub fn pattern_preview(pattern: &str) -> String {
    const MAX: usize = 80;
    if pattern.chars().count() > MAX {
        let head: String = pattern.chars().take(MAX).collect();
        format!("{head}…")
    } else {
        pattern.to_string()
    }
}

// ── Redis-backed cache ────────────────────────────────────────────────────────

#[cfg(feature = "redis")]
mod cache_entry {
    use serde::{Deserialize, Serialize};

    /// The JSON value stored in `cr:meta:{guild_id}` under field `{id}`.
    #[derive(Serialize, Deserialize)]
    pub struct CrEntry {
        pub pattern: String,
        pub anywhere: bool,
        pub image_url: String,
    }
}

#[cfg(feature = "redis")]
use cache_entry::CrEntry;

/// A matched reaction ready to send.
pub struct MatchedReaction {
    pub id: i64,
    pub image_url: String,
}

/// One live reaction with its stable internal id. The per-guild display number
/// is the 1-based position in this list ordered by id — assigned by the caller,
/// never stored. See [`live_ordered`].
pub struct ReactionEntry {
    pub id: i64,
    pub pattern: String,
    pub anywhere: bool,
    pub image_url: String,
}

/// Outcome of a `remove` keyed by per-guild number.
pub enum RemoveOutcome {
    /// Removed; carries the reaction's pattern.
    Removed(String),
    /// No reaction holds that number (out of range, or already gone).
    NotFound,
    /// A reaction holds that number but no longer matches the selected one — the
    /// list was renumbered between selection and submit.
    Changed,
}

/// Seed the cache from the DB on cold start (gated by `cr:seeded`).
#[cfg_attr(not(feature = "redis"), allow(unused_variables))]
#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?pool,
    )
)]
pub async fn init(pool: &PgPool) -> Result<(), Error> {
    #[cfg(feature = "redis")]
    {
        let Some(mut conn) = cache::conn().await else {
            return Ok(());
        };

        if cache::key_exists(&mut conn, CR_SEEDED_KEY).await? {
            return Ok(());
        }

        let rows = CustomReactionsTable::fetch_all_live(pool).await?;
        for (guild_id, row) in rows {
            let entry = CrEntry {
                pattern: row.pattern,
                anywhere: row.anywhere,
                image_url: row.image_url,
            };
            let json = serde_json::to_string(&entry)
                .map_err(|e| Error::from(format!("Failed to serialize cache entry: {e}")))?;
            cache::hash_set(&mut conn, &meta_key(guild_id), &row.id.to_string(), &json).await?;
            cache::set_add(&mut conn, CR_GUILDS_KEY, guild_id as u64).await?;
        }

        // Mark seeded.
        let _: () = redis::cmd("SET")
            .arg(CR_SEEDED_KEY)
            .arg("1")
            .query_async(&mut conn)
            .await?;
    }
    Ok(())
}

/// Rebuild a guild's Redis cache from the authoritative DB live `rows`, so the
/// cache can't drift from Postgres after a mutation. Runs as one MULTI/EXEC: the
/// hash is replaced and `cr:guilds` membership is updated in lockstep, so
/// `matching` never observes a half-built hash, a soft-deleted reaction left
/// behind by a missed delete, or a live guild missing from `cr:guilds`.
/// Best-effort — a Redis error is returned for the caller to log; the DB stays
/// authoritative and the next mutation (or a restart) reseeds.
#[cfg(feature = "redis")]
async fn reseed_guild_cache(
    conn: &mut redis::aio::ConnectionManager,
    guild_id: i64,
    rows: &[crate::enums::schemas::CustomReactionRow],
) -> Result<(), redis::RedisError> {
    let mut pipe = redis::pipe();
    pipe.atomic();
    pipe.cmd("DEL").arg(meta_key(guild_id)).ignore();
    if rows.is_empty() {
        pipe.cmd("SREM")
            .arg(CR_GUILDS_KEY)
            .arg(guild_id as u64)
            .ignore();
    } else {
        for row in rows {
            let entry = CrEntry {
                pattern: row.pattern.clone(),
                anywhere: row.anywhere,
                image_url: row.image_url.clone(),
            };
            match serde_json::to_string(&entry) {
                Ok(json) => {
                    pipe.cmd("HSET")
                        .arg(meta_key(guild_id))
                        .arg(row.id.to_string())
                        .arg(json)
                        .ignore();
                }
                Err(e) => {
                    tracing::warn!(error = %e, guild_id, id = row.id, "Skipping uncacheable reaction while reseeding");
                }
            }
        }
        pipe.cmd("SADD")
            .arg(CR_GUILDS_KEY)
            .arg(guild_id as u64)
            .ignore();
    }
    pipe.query_async::<()>(conn).await
}

/// Register a new reaction. Enforces the per-guild cap and writes through to both
/// Postgres and Redis.
#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?pool,
        guild_id = %guild_id,
        pattern = %pattern,
        anywhere = %anywhere,
    )
)]
pub async fn register(
    pool: &PgPool,
    guild_id: i64,
    pattern: &str,
    image_url: &str,
    anywhere: bool,
) -> Result<(i64, i64), Error> {
    const GUILD_CAP: i64 = 25;

    let count = CustomReactionsTable::count_live(pool, guild_id).await?;
    if count >= GUILD_CAP {
        return Err(Error::from(format!(
            "This server already has {GUILD_CAP} custom reactions (the maximum). \
             Remove one before adding another."
        )));
    }

    let id = CustomReactionsTable::insert(pool, guild_id, pattern, image_url, anywhere).await?;

    // Read the authoritative live set (now including the new row) once: it gives
    // both the per-guild number and the exact contents to refresh the cache with,
    // so the confirmation, `list`, and `remove` always agree on this reaction's
    // number. Falls back to count+1 only if the just-inserted row isn't read back.
    let rows = CustomReactionsTable::fetch_live(pool, guild_id).await?;
    let seq = rows
        .iter()
        .position(|r| r.id == id)
        .map_or(count + 1, |p| p as i64 + 1);

    #[cfg(feature = "redis")]
    if let Some(mut conn) = cache::conn().await
        && let Err(e) = reseed_guild_cache(&mut conn, guild_id, &rows).await
    {
        tracing::warn!(error = %e, guild_id, "Failed to refresh guild cache after register");
    }

    Ok((id, seq))
}

/// Live reactions for a guild, ordered by id — the single authoritative source
/// for per-guild numbering (position 1..N), shared by `list`, `remove`, the
/// remove autocomplete, and the `register` confirmation so every surface agrees.
///
/// Reads Postgres directly. The per-message firing path ([`matching`]) keeps its
/// Redis cache; these are infrequent staff commands, so the query cost is
/// irrelevant and reading DB truth avoids any list-vs-remove cache-drift
/// mismatch (a stale hash could otherwise number a soft-deleted row).
async fn live_ordered(pool: &PgPool, guild_id: i64) -> Vec<ReactionEntry> {
    CustomReactionsTable::fetch_live(pool, guild_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| ReactionEntry {
            id: r.id,
            pattern: r.pattern,
            anywhere: r.anywhere,
            image_url: r.image_url,
        })
        .collect()
}

/// Live reactions for a guild, ordered by id, for the `list` command. The
/// caller numbers them 1..N for display.
pub async fn list_live(pool: &PgPool, guild_id: i64) -> Vec<ReactionEntry> {
    live_ordered(pool, guild_id).await
}

/// Soft-delete the reaction at per-guild number `seq` (1-based, ordered by id).
///
/// `expected_preview`, when given, is the [`pattern_preview`] the caller showed
/// the user (carried in the autocomplete value). If the row now at `seq` no
/// longer matches it — because a concurrent add/remove renumbered the list
/// between selection and submit — the delete is refused with
/// [`RemoveOutcome::Changed`] rather than removing the wrong reaction. Also
/// evicts the compiled regex and updates the Redis cache.
#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?pool,
        guild_id = %guild_id,
        seq = %seq,
    )
)]
pub async fn remove(
    pool: &PgPool,
    guild_id: i64,
    seq: i64,
    expected_preview: Option<&str>,
) -> Result<RemoveOutcome, Error> {
    let Some(index) = seq.checked_sub(1).and_then(|i| usize::try_from(i).ok()) else {
        return Ok(RemoveOutcome::NotFound);
    };
    let entries = live_ordered(pool, guild_id).await;
    let Some(entry) = entries.get(index) else {
        return Ok(RemoveOutcome::NotFound);
    };
    if let Some(expected) = expected_preview
        && pattern_preview(&entry.pattern) != expected
    {
        return Ok(RemoveOutcome::Changed);
    }
    let id = entry.id;
    let pattern = entry.pattern.clone();

    if !CustomReactionsTable::soft_delete(pool, id, guild_id).await? {
        return Ok(RemoveOutcome::NotFound);
    }
    evict_compiled(id);

    // Rebuild the guild's cache from the post-delete live set so the removed
    // reaction can't linger in the hash (and `cr:guilds` is dropped when the
    // guild's last reaction is gone) — the cache stays in lockstep with the DB.
    #[cfg(feature = "redis")]
    if let Some(mut conn) = cache::conn().await {
        match CustomReactionsTable::fetch_live(pool, guild_id).await {
            Ok(rows) => {
                if let Err(e) = reseed_guild_cache(&mut conn, guild_id, &rows).await {
                    tracing::warn!(error = %e, guild_id, "Failed to refresh guild cache after remove");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, guild_id, "Failed to read live rows to refresh cache after remove");
            }
        }
    }

    Ok(RemoveOutcome::Removed(pattern))
}

/// Returns all reactions whose pattern matches `content` (sorted by id).
/// Uses the Redis hot path when available; falls back to the DB.
#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?pool,
        guild_id = %guild_id,
        content = %content,
    )
)]
pub async fn matching(
    pool: &PgPool,
    guild_id: i64,
    content: &str,
) -> Result<Vec<MatchedReaction>, Error> {
    #[cfg(feature = "redis")]
    {
        if let Some(mut conn) = cache::conn().await {
            // Short-circuit: if the guild isn't in cr:guilds it has no reactions.
            match cache::set_contains(&mut conn, CR_GUILDS_KEY, guild_id as u64).await {
                Ok(false) => return Ok(vec![]),
                Ok(true) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "cr:guilds SISMEMBER failed; falling through to DB");
                    return matching_from_db(pool, guild_id, content).await;
                }
            }

            match cache::hash_getall(&mut conn, &meta_key(guild_id)).await {
                Ok(pairs) => {
                    let mut results: Vec<MatchedReaction> = pairs
                        .iter()
                        .filter_map(|(field, value)| {
                            let id: i64 = field.parse().ok()?;
                            let entry: CrEntry = serde_json::from_str(value).ok()?;
                            let re = compiled_regex(id, &entry.pattern, entry.anywhere)?;
                            re.is_match(content).then_some(MatchedReaction {
                                id,
                                image_url: entry.image_url,
                            })
                        })
                        .collect();
                    results.sort_by_key(|r| r.id);
                    return Ok(results);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "HGETALL failed; falling through to DB");
                }
            }
        }
    }

    matching_from_db(pool, guild_id, content).await
}

/// DB fallback for `matching` — used when Redis is unavailable or errors.
async fn matching_from_db(
    pool: &PgPool,
    guild_id: i64,
    content: &str,
) -> Result<Vec<MatchedReaction>, Error> {
    let rows = CustomReactionsTable::fetch_live(pool, guild_id).await?;
    let mut results: Vec<MatchedReaction> = rows
        .into_iter()
        .filter_map(|row| {
            let re = compiled_regex(row.id, &row.pattern, row.anywhere)?;
            re.is_match(content).then_some(MatchedReaction {
                id: row.id,
                image_url: row.image_url,
            })
        })
        .collect();
    results.sort_by_key(|r| r.id);
    Ok(results)
}

/// Returns autocomplete entries for the `remove` subcommand. Each entry is
/// `"{seq} — {pattern}"`, where `seq` is the per-guild number (1-based position
/// ordered by id, the same numbering `list` shows). The number is the absolute
/// position in the full live set, so it still maps back correctly after the
/// partial filter narrows the visible rows.
pub async fn autocomplete_reactions(pool: &PgPool, guild_id: i64, partial: &str) -> Vec<String> {
    let needle = partial.to_lowercase();
    live_ordered(pool, guild_id)
        .await
        .iter()
        .enumerate()
        .filter(|(_, e)| needle.is_empty() || e.pattern.to_lowercase().contains(&needle))
        .take(MAX_AUTOCOMPLETE)
        .map(|(i, e)| format!("{} — {}", i + 1, pattern_preview(&e.pattern)))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Error>;

    // Sentinel guild IDs — each test gets its own to avoid parallel-test races.
    const CR_TEST_GUILD: i64 = 0x5AFE_0005_0000_0001;
    const CR_CAP_GUILD: i64 = 0x5AFE_0005_0000_0002;
    const CR_REDIS_GUILD: i64 = 0x5AFE_0005_0000_0003;
    const CR_MULTI_GUILD: i64 = 0x5AFE_0005_0000_0004;
    const CR_AC_GUILD: i64 = 0x5AFE_0005_0000_0005;
    const CR_DB_GUILD: i64 = 0x5AFE_0005_0000_0006;

    // ── Regex unit tests (no DB/Redis required) ───────────────────────────────

    #[test]
    fn anchored_full_match_only() {
        let re = compile_pattern("hello", false).unwrap();
        assert!(re.is_match("hello"));
        assert!(!re.is_match("say hello"));
        assert!(!re.is_match("hello world"));
    }

    #[test]
    fn anywhere_matches_substring() {
        let re = compile_pattern("hello", true).unwrap();
        assert!(re.is_match("say hello there"));
        assert!(re.is_match("hello"));
        assert!(!re.is_match("world"));
    }

    #[test]
    fn invalid_pattern_rejected() {
        // Unbalanced group and a backwards repetition both fail to compile and
        // surface the human-readable error.
        assert!(compile_pattern("(", false).is_err());
        assert!(compile_pattern("a{2,1}", true).is_err());
    }

    #[test]
    fn case_insensitive_via_inline_flag() {
        let re = compile_pattern("(?i)hello", false).unwrap();
        assert!(re.is_match("HELLO"));
        assert!(re.is_match("Hello"));
    }

    #[test]
    fn empty_match_rejected() {
        assert!(compile_pattern(".*", false).is_err());
        assert!(compile_pattern("a?", false).is_err());
        assert!(compile_pattern("", false).is_err());
        assert!(compile_pattern(".*", true).is_err());
    }

    #[test]
    fn oversized_pattern_rejected() {
        let pat = "a".repeat(PATTERN_MAX_LEN + 1);
        assert!(compile_pattern(&pat, false).is_err());
    }

    #[test]
    fn url_validation_rejects_page_links_only() {
        // Tenor/Giphy page links are HTML, not images — rejected.
        assert!(validate_image_url("https://tenor.com/view/cat-dance-gif-12345").is_err());
        assert!(validate_image_url("https://www.tenor.com/view/x").is_err());
        assert!(validate_image_url("https://giphy.com/gifs/funny-abc123").is_err());
        assert!(validate_image_url("https://www.giphy.com/gifs/x").is_err());
        // Giphy clip/sticker pages too.
        assert!(validate_image_url("https://giphy.com/clips/x").is_err());
        assert!(validate_image_url("https://giphy.com/stickers/x").is_err());
        // No trailing slash, double slash, and a port must still be caught.
        assert!(validate_image_url("https://tenor.com/view").is_err());
        assert!(validate_image_url("https://giphy.com/gifs").is_err());
        assert!(validate_image_url("https://tenor.com//view/x").is_err());
        assert!(validate_image_url("https://tenor.com:443/view/x").is_err());
        // Non-http(s) input is rejected outright.
        assert!(validate_image_url("ftp://host/a.gif").is_err());
        assert!(validate_image_url("not a url").is_err());

        // Direct media URLs and anything else pass through.
        assert!(validate_image_url("https://media1.tenor.com/m/ID/slug.gif").is_ok());
        assert!(validate_image_url("https://media.giphy.com/media/ID/giphy.gif").is_ok());
        assert!(validate_image_url("https://cdn.discordapp.com/attachments/1/2/a.png").is_ok());
        assert!(validate_image_url("https://example.com/whatever").is_ok());
        // The Tenor search/home page (no /view/) isn't our page-link pattern.
        assert!(validate_image_url("https://tenor.com/search/cat-gifs").is_ok());
    }

    // ── DB roundtrip (skips when DATABASE_URL absent) ─────────────────────────

    use crate::tests::test_pool;

    async fn cleanup(pool: &PgPool) -> TestResult {
        sqlx::query("DELETE FROM custom_reactions WHERE guild_id = $1")
            .bind(CR_TEST_GUILD)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn register_remove_matching_roundtrip() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        cleanup(&pool).await?;

        let (id, seq) = register(
            &pool,
            CR_TEST_GUILD,
            "ping",
            "https://example.com/img.gif",
            false,
        )
        .await?;
        // First reaction in a fresh guild is per-guild number 1.
        assert_eq!(seq, 1);

        // Must match the full trimmed content (anchored).
        let hits = matching(&pool, CR_TEST_GUILD, "ping").await?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);

        // Must not match a substring.
        assert!(matching(&pool, CR_TEST_GUILD, "say ping").await?.is_empty());

        // A stale preview (the row no longer matches) is refused, not deleted.
        assert!(matches!(
            remove(&pool, CR_TEST_GUILD, seq, Some("stale")).await?,
            RemoveOutcome::Changed
        ));
        assert!(!matching(&pool, CR_TEST_GUILD, "ping").await?.is_empty());

        // Remove by per-guild number; the removed pattern comes back.
        assert!(matches!(
            remove(&pool, CR_TEST_GUILD, seq, Some("ping")).await?,
            RemoveOutcome::Removed(p) if p == "ping"
        ));
        assert!(matching(&pool, CR_TEST_GUILD, "ping").await?.is_empty());
        // A second remove of the same number now finds nothing.
        assert!(matches!(
            remove(&pool, CR_TEST_GUILD, seq, None).await?,
            RemoveOutcome::NotFound
        ));

        cleanup(&pool).await?;
        Ok(())
    }

    /// A message matching several reactions fires all of them, ordered by id,
    /// and leaves non-matching ones out.
    #[tokio::test]
    async fn multi_match_fires_all_in_id_order() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        reset_guild(&pool, CR_MULTI_GUILD).await?;

        let (foo, foo_seq) = register(
            &pool,
            CR_MULTI_GUILD,
            "foo",
            "https://example.com/foo.gif",
            true,
        )
        .await?;
        let (bar, bar_seq) = register(
            &pool,
            CR_MULTI_GUILD,
            "bar",
            "https://example.com/bar.gif",
            true,
        )
        .await?;
        // Non-matching for the probe message below.
        let (_baz, baz_seq) = register(
            &pool,
            CR_MULTI_GUILD,
            "baz",
            "https://example.com/baz.gif",
            true,
        )
        .await?;
        // Per-guild numbers increment in registration order.
        assert_eq!((foo_seq, bar_seq, baz_seq), (1, 2, 3));

        let hits = matching(&pool, CR_MULTI_GUILD, "foo and bar").await?;
        let ids: Vec<i64> = hits.iter().map(|h| h.id).collect();
        // Both matching reactions fire, in ascending id order; baz is excluded.
        assert_eq!(ids, vec![foo, bar]);
        assert!(foo < bar);

        reset_guild(&pool, CR_MULTI_GUILD).await?;
        Ok(())
    }

    /// `matching_from_db` (the Redis-down fallback) matches anchored against the
    /// live DB rows, independent of cache state.
    #[tokio::test]
    async fn matching_from_db_anchored_fallback() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        reset_guild(&pool, CR_DB_GUILD).await?;

        let id = CustomReactionsTable::insert(
            &pool,
            CR_DB_GUILD,
            "ping",
            "https://example.com/p.gif",
            false,
        )
        .await?;

        let hits = matching_from_db(&pool, CR_DB_GUILD, "ping").await?;
        assert_eq!(hits.iter().map(|h| h.id).collect::<Vec<_>>(), vec![id]);
        // Anchored: a substring must not match.
        assert!(
            matching_from_db(&pool, CR_DB_GUILD, "say ping")
                .await?
                .is_empty()
        );

        reset_guild(&pool, CR_DB_GUILD).await?;
        Ok(())
    }

    /// Autocomplete lists live reactions as `"{seq} — {pattern}"`, where `seq`
    /// is the per-guild number (1-based, ordered by id), filtered
    /// case-insensitively by the partial input. The number is the absolute
    /// position, so a filtered result keeps the original numbers.
    #[tokio::test]
    async fn autocomplete_lists_and_filters() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        reset_guild(&pool, CR_AC_GUILD).await?;

        // Registered in this order, so apple=1, apricot=2, banana=3.
        for (pat, file) in [("apple", "a"), ("apricot", "b"), ("banana", "c")] {
            register(
                &pool,
                CR_AC_GUILD,
                pat,
                &format!("https://example.com/{file}.gif"),
                true,
            )
            .await?;
        }

        // Empty partial lists all three, ordered by id, in the display format.
        let all = autocomplete_reactions(&pool, CR_AC_GUILD, "").await;
        assert_eq!(all, vec!["1 — apple", "2 — apricot", "3 — banana"]);

        // Case-insensitive prefix filter narrows to the two "ap*" patterns,
        // keeping their absolute per-guild numbers.
        let ap = autocomplete_reactions(&pool, CR_AC_GUILD, "AP").await;
        assert_eq!(ap, vec!["1 — apple", "2 — apricot"]);

        reset_guild(&pool, CR_AC_GUILD).await?;
        Ok(())
    }

    /// Clear both the DB rows and any cached Redis state for a guild, so a
    /// crashed prior run can't leave stale entries that skew a fresh run.
    async fn reset_guild(pool: &PgPool, guild_id: i64) -> TestResult {
        sqlx::query("DELETE FROM custom_reactions WHERE guild_id = $1")
            .bind(guild_id)
            .execute(pool)
            .await?;
        #[cfg(feature = "redis")]
        if let Some(mut conn) = cache::conn().await {
            let _: () = redis::cmd("DEL")
                .arg(&meta_key(guild_id))
                .query_async(&mut conn)
                .await
                .unwrap_or(());
            let _: () = redis::cmd("SREM")
                .arg(CR_GUILDS_KEY)
                .arg(guild_id as u64)
                .query_async(&mut conn)
                .await
                .unwrap_or(());
        }
        Ok(())
    }

    async fn cleanup_guild(pool: &PgPool, guild_id: i64) -> TestResult {
        sqlx::query("DELETE FROM custom_reactions WHERE guild_id = $1")
            .bind(guild_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn guild_cap_enforced() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        cleanup_guild(&pool, CR_CAP_GUILD).await?;

        // Insert 25 reactions directly (bypass the cap check in `register`).
        for i in 0..25i64 {
            CustomReactionsTable::insert(
                &pool,
                CR_CAP_GUILD,
                &format!("pat{i}"),
                "https://example.com/x.gif",
                false,
            )
            .await?;
        }

        // 26th via `register` must be rejected.
        let result = register(
            &pool,
            CR_CAP_GUILD,
            "overflow",
            "https://example.com/x.gif",
            false,
        )
        .await;
        assert!(result.is_err());

        cleanup_guild(&pool, CR_CAP_GUILD).await?;
        Ok(())
    }

    // ── Redis cache roundtrip (skips when REDIS_URL absent) ───────────────────

    #[cfg(feature = "redis")]
    use crate::tests::test_redis;

    #[cfg(feature = "redis")]
    #[tokio::test]
    async fn cache_init_and_matching_via_redis() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        let Some(mut conn) = test_redis().await else {
            return Ok(());
        };
        cleanup_guild(&pool, CR_REDIS_GUILD).await?;

        // Reset sentinel so init actually seeds.
        let _: () = redis::cmd("DEL")
            .arg(CR_SEEDED_KEY)
            .query_async(&mut conn)
            .await
            .unwrap_or(());
        let _: () = redis::cmd("DEL")
            .arg(&meta_key(CR_REDIS_GUILD))
            .query_async(&mut conn)
            .await
            .unwrap_or(());
        let _: () = redis::cmd("SREM")
            .arg(CR_GUILDS_KEY)
            .arg(CR_REDIS_GUILD as u64)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let id = CustomReactionsTable::insert(
            &pool,
            CR_REDIS_GUILD,
            "pong",
            "https://example.com/pong.gif",
            false,
        )
        .await?;

        init(&pool).await?;

        // Cache should now serve the match.
        let hits = matching(&pool, CR_REDIS_GUILD, "pong").await?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);

        // No match for substring.
        assert!(
            matching(&pool, CR_REDIS_GUILD, "say pong")
                .await?
                .is_empty()
        );

        // Cleanup Redis state.
        let _: () = redis::cmd("DEL")
            .arg(CR_SEEDED_KEY)
            .query_async(&mut conn)
            .await
            .unwrap_or(());
        let _: () = redis::cmd("DEL")
            .arg(&meta_key(CR_REDIS_GUILD))
            .query_async(&mut conn)
            .await
            .unwrap_or(());
        let _: () = redis::cmd("SREM")
            .arg(CR_GUILDS_KEY)
            .arg(CR_REDIS_GUILD as u64)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        cleanup_guild(&pool, CR_REDIS_GUILD).await?;
        Ok(())
    }
}
