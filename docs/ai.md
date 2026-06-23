# AI features

An operator and user guide for the optional AI features: the in-character Hu
Tao persona chat, auto-reply channels, and the conversation context window.

## Enabling AI at build time

AI is compiled in via Cargo features. `ai` is a meta-feature: it also enables
`redis`, and on its own it does not pick a provider. You must enable exactly
one backend:

```
ai-deepseek  ai-ollama  ai-anthropic  ai-openai  ai-google  ai-groq  ai-openrouter
```

Building with `--features ai` and no backend fails with a `compile_error!`.

```sh
cargo run --release --features ai-deepseek
```

Set `AI_MODEL` (required when AI is built) and, for hosted backends,
`AI_API_KEY`. `ai-ollama` does not need a key and can point at a custom base
URL with `AI_CHAT_ENDPOINT`. See [configuration](./configuration.md) for the
full variable list.

## Persona chat

Provider-agnostic, plain chat completion through the `llm` crate. No tool
calling.

- `/ai prompt:<text>` — a one-off prompt in any channel or DM.
- `/aichannel` — toggle a channel where the bot auto-replies to every message.
  Requires Manage Channels. Registered channels survive restarts.

Persona chat also runs in DMs.

### Per-guild custom prompt

`/custom prompt` appends a server-specific instruction block to the shared Hu
Tao system prompt without replacing the persona. Requires Manage Channels.

- `/custom prompt add <text>` — set or replace the guild's extra prompt.
- `/custom prompt show` — show the current one (ephemeral).
- `/custom prompt remove` — clear it.

The extra prompt is injected as a leading turn ahead of the conversation, so it
applies to both `/ai` and `/aichannel` auto-replies. It is stored in
`guild_ai_settings` and cached in Redis (`ai:guild_prompt:{guild}`, TTL 1800s,
DB authoritative); the no-prompt case is negative-cached so the per-message path
stays off the DB. DMs have no guild, so they use the base persona only.

### Context window

Conversation context is kept per channel so the persona can follow a thread.
The behavior depends on whether Redis is configured.

- With `REDIS_URL` set, recent turns are stored as a windowed Redis list
  (`ai:ctx:{channel}`, TTL 1800s), sized by `AI_MAX_MSG_CONTEXT` (default 10,
  cap 100).
- Without Redis, the bot re-fetches recent messages from Discord on every
  reply instead.

Three details worth knowing:

- **Warm channels only.** `record_message` appends a message to the window only
  if the window already exists. Channels become "warm" after a prior AI
  interaction; a cold channel is seeded on demand by a one-off Discord fetch
  the first time the bot needs context there.
- **Embeds are flattened.** Most command outputs are embed-only with empty
  message `content`. The context layer renders embeds to text and appends them,
  so the model sees command output instead of nothing.
- **Replies carry their parent.** When a message is an inline reply, its
  rendered turn is prefixed with `[replying to <author>: <snippet>]`, where the
  snippet is the parent's *body* (content + embeds) flattened and truncated the
  same way — but **not** the parent's own reply marker. This is what keeps it
  strictly one level deep: snippeting the parent's fully-rendered turn would
  re-include the parent's marker and nest unbounded down the reply chain. A
  deleted or unresolved parent is skipped. The bot's *own* turns never carry the
  marker — it's a cue for reading other people's reply links, and on the
  `assistant` turns it only trains the model to begin its replies with
  `[replying to ...]` and parrot the marker into the visible message. As a
  belt-and-braces guard, any marker the model still echoes is stripped from the
  response before it's sent.

Speakers are distinguished in the window: the bot's own messages become
`assistant` turns, and everyone else becomes a `user` turn prefixed with their
display name.

### Rate limits and locks

- Per-user AI rate limit: 10s (`ai:rl:{user}`), Redis-backed. Without Redis the
  bot never rate-limits.
- Per-channel AI lock: 30s (`ai:ch_lock:{ch}`), to dedupe concurrent replies in
  one channel. Without Redis this is a no-op.

These are separate from the XP levelling cooldown (~60s), which is an unrelated
system.