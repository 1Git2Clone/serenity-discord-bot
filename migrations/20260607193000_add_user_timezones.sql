CREATE TABLE user_timezones (
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL, -- 0 = the user's global default (no real guild id is 0)
    timezone TEXT NOT NULL,
    PRIMARY KEY (user_id, guild_id)
);
