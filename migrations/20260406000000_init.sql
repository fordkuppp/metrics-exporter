CREATE TABLE IF NOT EXISTS steam_sessions (
    steam_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    game_name TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ
) WITH (
    tsdb.hypertable
);

CREATE INDEX IF NOT EXISTS idx_steam_sessions_steam_id ON steam_sessions (steam_id, started_at DESC);

CREATE TABLE IF NOT EXISTS steam_player_summaries (
    polled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    steam_id TEXT NOT NULL,
    persona_name TEXT,
    profile_url TEXT NOT NULL,
    avatar TEXT,
    avatar_medium TEXT,
    avatar_full TEXT,
    persona_state SMALLINT NOT NULL DEFAULT 0,
    community_visibility_state SMALLINT NOT NULL DEFAULT 1,
    profile_state BOOLEAN NOT NULL DEFAULT FALSE,
    last_logoff BIGINT,
    comment_permission BOOLEAN NOT NULL DEFAULT FALSE,
    real_name TEXT,
    primary_clan_id TEXT,
    time_created BIGINT,
    game_id TEXT,
    game_server_ip TEXT,
    game_extra_info TEXT,
    loc_country_code TEXT,
    loc_state_code TEXT,
    loc_city_id INTEGER
) WITH (
    tsdb.hypertable
);

CREATE INDEX IF NOT EXISTS idx_steam_player_summaries_steam_id ON steam_player_summaries (steam_id, polled_at DESC);
