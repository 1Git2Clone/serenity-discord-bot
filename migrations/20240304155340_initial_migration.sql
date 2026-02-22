-- Add migration script here
CREATE TABLE user_stats (
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    experience_points INTEGER NOT NULL,
    level INTEGER NOT NULL,
    PRIMARY KEY (user_id, guild_id)
);

CREATE INDEX idx_user_id ON user_stats (user_id);
CREATE INDEX idx_guild_id ON user_stats (guild_id);
CREATE INDEX idx_user_guild ON user_stats (user_id, guild_id);

CREATE TABLE bot_mentions (
    mentions BIGINT NOT NULL
);
CREATE INDEX idx_mentions ON bot_mentions (mentions);
INSERT INTO bot_mentions (mentions) VALUES (0);
