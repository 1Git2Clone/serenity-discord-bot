# Custom reactions

Per-guild image reactions: staff register an image plus a Rust regex, and any
guild message matching the pattern gets a red embed reply with the image.
Commands are under `/custom reaction` (`add`, `list`, `remove`), all gated by
Manage Channels. The user-facing command reference is in the
[README](../README.md#features); this page covers the parts that are not obvious
from the command list.

Code: `src/data/custom_reactions.rs` and
`src/commands/embed_commands/spec/custom_reaction.rs`.

## Per-guild numbering

Reactions are shown with a per-guild number (`#1`, `#2`, … within the server),
not the global database id.

- The number is **positional**: the 1-based index into the guild's live
  reactions ordered by id. It is computed in Rust (`live_ordered`) and **never
  stored**.
- The global database id stays the internal key — it keys the compiled-regex
  cache and the Redis hash field.
- All four surfaces — the add confirmation, `list`, the remove autocomplete, and
  remove itself — derive the number from the same DB-ordered source, so they
  never disagree.

Because the number is positional, removing a reaction renumbers the rest.
`list` always reflects the current numbers.

## Removing safely

`remove` targets a per-guild number, which is fragile if the list was renumbered
between the moment the user opened the autocomplete and the moment they
submitted. Two things guard against deleting the wrong reaction:

- The autocomplete value is `"{seq} — {preview}"`, carrying an 80-char pattern
  preview (`pattern_preview`) alongside the number.
- `remove` looks up the row at that number and compares its preview to the one
  that was selected. The result is a `RemoveOutcome`:
  - `Removed(pattern)` — deleted; the pattern is echoed back.
  - `NotFound` — no reaction holds that number (out of range or already gone).
  - `Changed` — a reaction holds the number but no longer matches the selected
    preview, so the list was renumbered. Nothing is deleted; the user is told to
    re-run `list`.

A plain typed number has no preview and skips the match check.

## Image-URL validation

`validate_image_url` rejects two things at register time:

- Non-`http(s)` input.
- Tenor and Giphy **page** links — `tenor.com/view/…`, `giphy.com/gifs/…`,
  `giphy.com/clips/…`, `giphy.com/stickers/…`. These are HTML pages, not media
  files. Discord renders them inline only when a user types them, never in a
  bot embed's image field, so they would show up as a blank box. The error
  points the user at the direct media URL (`media.tenor.com/….gif`,
  `media.giphy.com/….gif`).

Matching is on the host and the first path segment, so the `:port`,
double-slash, and no-trailing-slash forms are all caught. Direct media URLs,
Discord CDN URLs, and everything else pass through. Discord CDN attachment URLs
have their signing query params stripped at storage time; external URLs are
stored as-is.

## Write-through Redis cache

Reads and writes route through a write-through Redis cache so the per-message
match check does not hit Postgres on every message. The cache is best-effort;
the database is always authoritative, and everything degrades to direct DB
queries when Redis is unavailable. See the
[Redis fallback model](./architecture.md#redis-optional-fallback-model).

Keys:

- `cr:guilds` — a set of guild ids that have at least one live reaction. The
  per-message handler checks this first: a guild not in the set short-circuits
  with no further lookup, which is the common case.
- `cr:meta:{guild_id}` — a hash of `id → entry JSON` for that guild's
  reactions.
- `cr:seeded` — a gate so the cold-start population from the database
  (`fetch_all_live`) runs once.

### Reseed on mutation

After any mutation, `reseed_guild_cache` rebuilds the guild's cache from the
authoritative DB rows in a single `MULTI`/`EXEC`: it replaces the
`cr:meta:{guild_id}` hash and updates `cr:guilds` membership in lockstep. Doing
it atomically means the matcher never observes a half-built hash, a
soft-deleted reaction left behind by a missed delete, or a live guild missing
from `cr:guilds`. A Redis error is returned for the caller to log; the next
mutation or a restart reseeds.
