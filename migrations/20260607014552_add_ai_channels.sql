CREATE TABLE ai_channels (
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    PRIMARY KEY (channel_id)
);

CREATE INDEX idx_ai_channels_guild_id ON ai_channels (guild_id);
