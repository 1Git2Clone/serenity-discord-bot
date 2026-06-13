CREATE TABLE custom_reactions (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    guild_id    BIGINT      NOT NULL,
    pattern     TEXT        NOT NULL,
    image_url   TEXT        NOT NULL,
    anywhere    BOOLEAN     NOT NULL DEFAULT FALSE,
    deleted_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_custom_reactions_guild_live
    ON custom_reactions (guild_id) WHERE deleted_at IS NULL;
