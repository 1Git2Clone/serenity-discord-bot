-- Add migration script here
CREATE TABLE user_stats (
  user_id INTEGER NOT NULL,
  guild_id INTEGER NOT NULL,
  experience_points INTEGER NOT NULL,
  level INTEGER NOT NULL,
  last_query_timestamp INTEGER NOT NULL,
  PRIMARY KEY (user_id, guild_id)
);
