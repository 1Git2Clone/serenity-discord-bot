CREATE TABLE guild_ai_settings (
    guild_id     BIGINT PRIMARY KEY,
    extra_prompt TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
