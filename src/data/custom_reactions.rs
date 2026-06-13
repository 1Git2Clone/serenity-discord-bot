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
) -> Result<i64, Error> {
    const GUILD_CAP: i64 = 25;

    let count = CustomReactionsTable::count_live(pool, guild_id).await?;
    if count >= GUILD_CAP {
        return Err(Error::from(format!(
            "This server already has {GUILD_CAP} custom reactions (the maximum). \
             Remove one before adding another."
        )));
    }

    let id = CustomReactionsTable::insert(pool, guild_id, pattern, image_url, anywhere).await?;

    #[cfg(feature = "redis")]
    {
        if let Some(mut conn) = cache::conn().await {
            let entry = CrEntry {
                pattern: pattern.to_string(),
                anywhere,
                image_url: image_url.to_string(),
            };
            match serde_json::to_string(&entry) {
                Ok(json) => {
                    if let Err(e) =
                        cache::hash_set(&mut conn, &meta_key(guild_id), &id.to_string(), &json)
                            .await
                    {
                        tracing::warn!(error = %e, guild_id, "Failed to write reaction to Redis hash");
                    }
                    if let Err(e) = cache::set_add(&mut conn, CR_GUILDS_KEY, guild_id as u64).await
                    {
                        tracing::warn!(error = %e, guild_id, "Failed to add guild to cr:guilds");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to serialize cache entry");
                }
            }
        }
    }

    Ok(id)
}

/// Soft-delete a reaction by id + guild. Removes from Redis if the guild has no
/// live reactions remaining.
#[tracing::instrument(
    fields(
        category = "sql",
        db_pool = ?pool,
        id = %id,
        guild_id = %guild_id,
    )
)]
pub async fn remove(pool: &PgPool, id: i64, guild_id: i64) -> Result<bool, Error> {
    let deleted = CustomReactionsTable::soft_delete(pool, id, guild_id).await?;

    if deleted {
        evict_compiled(id);
    }

    #[cfg(feature = "redis")]
    if deleted && let Some(mut conn) = cache::conn().await {
        if let Err(e) = cache::hash_del(&mut conn, &meta_key(guild_id), &id.to_string()).await {
            tracing::warn!(error = %e, guild_id, id, "Failed to remove reaction from Redis hash");
        }
        // If no live reactions remain, drop the guild from cr:guilds.
        match CustomReactionsTable::count_live(pool, guild_id).await {
            Ok(0) => {
                if let Err(e) = cache::set_remove(&mut conn, CR_GUILDS_KEY, guild_id as u64).await {
                    tracing::warn!(error = %e, guild_id, "Failed to remove guild from cr:guilds");
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, guild_id, "Failed to count live reactions after remove");
            }
        }
    }

    Ok(deleted)
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

/// Returns autocomplete entries for the `remove` subcommand.
/// Each entry is `"{id} — {pattern}"` (pattern truncated to 80 chars).
/// Served from the Redis cache when available.
pub async fn autocomplete_reactions(pool: &PgPool, guild_id: i64, partial: &str) -> Vec<String> {
    let needle = partial.to_lowercase();

    #[cfg(feature = "redis")]
    if let Some(mut conn) = cache::conn().await
        && let Ok(pairs) = cache::hash_getall(&mut conn, &meta_key(guild_id)).await
    {
        let mut entries: Vec<(i64, String)> = pairs
            .iter()
            .filter_map(|(field, value)| {
                let id: i64 = field.parse().ok()?;
                let entry: CrEntry = serde_json::from_str(value).ok()?;
                (needle.is_empty() || entry.pattern.to_lowercase().contains(&needle))
                    .then_some((id, entry.pattern))
            })
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        return entries
            .into_iter()
            .take(MAX_AUTOCOMPLETE)
            .map(|(id, pat)| {
                let preview: String = pat.chars().take(80).collect();
                format!("{id} — {preview}")
            })
            .collect();
    }

    // DB fallback.
    let rows = CustomReactionsTable::fetch_live(pool, guild_id)
        .await
        .unwrap_or_default();
    rows.into_iter()
        .filter(|r| needle.is_empty() || r.pattern.to_lowercase().contains(&needle))
        .take(MAX_AUTOCOMPLETE)
        .map(|r| {
            let preview: String = r.pattern.chars().take(80).collect();
            format!("{} — {preview}", r.id)
        })
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

        let id = register(
            &pool,
            CR_TEST_GUILD,
            "ping",
            "https://example.com/img.gif",
            false,
        )
        .await?;

        // Must match the full trimmed content (anchored).
        let hits = matching(&pool, CR_TEST_GUILD, "ping").await?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);

        // Must not match a substring.
        assert!(matching(&pool, CR_TEST_GUILD, "say ping").await?.is_empty());

        assert!(remove(&pool, id, CR_TEST_GUILD).await?);
        assert!(matching(&pool, CR_TEST_GUILD, "ping").await?.is_empty());

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

        let foo = register(
            &pool,
            CR_MULTI_GUILD,
            "foo",
            "https://example.com/foo.gif",
            true,
        )
        .await?;
        let bar = register(
            &pool,
            CR_MULTI_GUILD,
            "bar",
            "https://example.com/bar.gif",
            true,
        )
        .await?;
        // Non-matching for the probe message below.
        let _baz = register(
            &pool,
            CR_MULTI_GUILD,
            "baz",
            "https://example.com/baz.gif",
            true,
        )
        .await?;

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

    /// Autocomplete lists live reactions as `"{id} — {pattern}"`, ordered by id,
    /// filtered case-insensitively by the partial input.
    #[tokio::test]
    async fn autocomplete_lists_and_filters() -> TestResult {
        let Some(pool) = test_pool().await else {
            return Ok(());
        };
        reset_guild(&pool, CR_AC_GUILD).await?;

        let apple = register(
            &pool,
            CR_AC_GUILD,
            "apple",
            "https://example.com/a.gif",
            true,
        )
        .await?;
        let apricot = register(
            &pool,
            CR_AC_GUILD,
            "apricot",
            "https://example.com/b.gif",
            true,
        )
        .await?;
        let _banana = register(
            &pool,
            CR_AC_GUILD,
            "banana",
            "https://example.com/c.gif",
            true,
        )
        .await?;

        // Empty partial lists all three, ordered by id, in the display format.
        let all = autocomplete_reactions(&pool, CR_AC_GUILD, "").await;
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], format!("{apple} — apple"));

        // Case-insensitive prefix filter narrows to the two "ap*" patterns.
        let ap = autocomplete_reactions(&pool, CR_AC_GUILD, "AP").await;
        assert_eq!(
            ap,
            vec![format!("{apple} — apple"), format!("{apricot} — apricot")]
        );

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
