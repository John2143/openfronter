-- Initial Migration

CREATE SCHEMA IF NOT EXISTS public;

-- Lobby Listeners

CREATE TABLE IF NOT EXISTS lobbies (
    game_id CHAR(8) PRIMARY KEY,
    teams INTEGER NOT NULL,
    max_players INTEGER NOT NULL,
    game_map TEXT NOT NULL,
    approx_num_players INTEGER NOT NULL,
    first_seen_unix_sec bigint NOT NULL,
    last_seen_unix_sec bigint NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    lobby_config_json JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS finished_games (
    game_id CHAR(8) PRIMARY KEY,
    result_json JSONB, -- If null, then the game has been moved to s3
    is_ok BOOLEAN NOT NULL DEFAULT TRUE,
    inserted_at_unix_sec bigint NOT NULL DEFAULT extract(epoch from NOW())
);

-- Game Analysis

CREATE SCHEMA IF NOT EXISTS analysis_1;

CREATE TABLE analysis_1.completed_analysis (
   game_id CHAR(8) NOT NULL PRIMARY KEY,
   inserted_at_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
   analysis_engine_version TEXT NOT NULL,
   FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE
);

CREATE TYPE analysis_1.player_type AS ENUM (
   'BOT',
   'FAKEHUMAN',
   'HUMAN'
);

CREATE TABLE analysis_1.players (
   game_id CHAR(8) NOT NULL,
   id CHAR(8) NOT NULL,
   client_id CHAR(8),
   small_id SMALLINT NOT NULL,
   player_type analysis_1.player_type NOT NULL,
   name TEXT NOT NULL,
   flag TEXT,
   team SMALLINT,
   FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE,
   PRIMARY KEY (game_id, id)
);

CREATE TABLE analysis_1.display_events (
  game_id CHAR(8) NOT NULL,
  tick SMALLINT NOT NULL,
  message_type TEXT NOT NULL,
  message TEXT NOT NULL,
  player_id SMALLINT NOT NULL,
  gold_amount INTEGER,
  FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE
);

CREATE TYPE analysis_1.event_type AS ENUM (
  'Tile',
  'Unit',
  'Player',
  'DisplayEvent',
  'DisplayChatEvent',
  'AllianceRequest',
  'AllianceRequestReply',
  'BrokeAlliance',
  'AllianceExpired',
  'AllianceExtension',
  'TargetPlayer',
  'Emoji',
  'Win',
  'Hash',
  'UnitIncoming',
  'BonusEvent',
  'RailroadEvent'
);

CREATE TABLE analysis_1.general_events (
  game_id CHAR(8) NOT NULL,
  tick SMALLINT NOT NULL,
  event_type analysis_1.event_type NOT NULL,
  data JSONB NOT NULL,
  FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE
);

CREATE TABLE analysis_1.spawn_locations (
    game_id CHAR(8) NOT NULL,
    tick SMALLINT NOT NULL,
    client_id CHAR(8) NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    previous_spawns JSONB DEFAULT '[]',
    PRIMARY KEY (game_id, client_id),
    FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE
);


-- These tables are a follow up from the original analysis_1.player_updates.
-- The goal is to store less data.

-- one game = 3 stages @ 10 ticks per second
---   - 50 player for 10 minutes:
--         about 50 players, 1 update per second, 6000 ticks = 300,000 rows
--    - 10 players for 20 minutes
--         about 10 players, 1 update per second, 1200 ticks = 12,000 rows
--    - 3 players for 30 minutes
--         about 3 players, 1 update per second, 1800 ticks = 5,400 rows
-- 8+2+2+2+2+2+2= = 19/20 bytes per row per update
-- 300,000 * 20b = 6,000,000 bytes = 6MB per game ish of analysis data
CREATE TABLE analysis_1.packed_player_updates (
   game_id CHAR(8) NOT NULL, -- 8 bytes
   small_id SMALLINT NOT NULL, -- 2 bytes (could be BIT(12)? would that be faster?)
   tick SMALLINT NOT NULL, -- 2 bytes
   player_alive BIT(1) NOT NULL, -- 1 bit
   player_connected BIT(1) NOT NULL, -- 1 bit
   tiles_owned SMALLINT NOT NULL DEFAULT 0, -- 2 bytes
   gold  SMALLINT NOT NULL DEFAULT 0, -- 2 bytes
   workers SMALLINT NOT NULL DEFAULT 0, -- 2 bytes
   troops SMALLINT NOT NULL DEFAULT 0, -- 2 bytes
   FOREIGN KEY (game_id) REFERENCES public.finished_games(game_id) ON DELETE CASCADE,
   PRIMARY KEY (game_id, tick, small_id)
);

-- When a user changes their target troop ratio, we want to store that. Cause
-- it's uncommon, we don't want to store it for every frame like before
CREATE TABLE analysis_1.troop_ratio_change (
   game_id CHAR(8) NOT NULL,
   small_id SMALLINT NOT NULL,
   client_id CHAR(8) NOT NULL,
   target_troop_ratio REAL NOT NULL
);

