CREATE SCHEMA social;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create a function to generate a random user ID
-- From stackoverflow
CREATE OR REPLACE FUNCTION social.generate_user_uid(size INT) RETURNS TEXT AS $$
DECLARE
  characters TEXT := 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  bytes BYTEA := gen_random_bytes(size);
  l INT := length(characters);
  i INT := 0;
  output TEXT := '';
BEGIN
  WHILE i < size LOOP
    output := output || substr(characters, get_byte(bytes, i) % l + 1, 1);
    i := i + 1;
  END LOOP;
  RETURN output;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE TABLE IF NOT EXISTS social.discord_link (
   user_id CHAR(10) NOT NULL PRIMARY KEY,
   discord_user_id TEXT NOT NULL UNIQUE,
   discord_username TEXT NOT NULL,
   discord_avatar TEXT,
   discord_global_name TEXT,
   created_at_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE TABLE IF NOT EXISTS social.registered_users (
    id CHAR(10) NOT NULL PRIMARY KEY DEFAULT social.generate_user_uid(10),
    username TEXT NOT NULL UNIQUE,
    openfront_player_id TEXT UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS social.friends (
    user_id CHAR(10) NOT NULL,
    friend_id CHAR(10) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, friend_id),
    FOREIGN KEY (user_id) REFERENCES social.registered_users(id) ON DELETE CASCADE,
    FOREIGN KEY (friend_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
);

-- "Alliances" in webapp
CREATE TABLE IF NOT EXISTS social.friend_requests (
    id SERIAL PRIMARY KEY,
    sender_id CHAR(10) NOT NULL,
    receiver_id CHAR(10) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'declined')),
    FOREIGN KEY (sender_id) REFERENCES social.registered_users(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
);

CREATE TYPE analysis_queue_status AS ENUM (
    'Pending',
    'Running',
    'Completed',
    'NotFound',
    'Failed',
    'Stalled',
    'Cancelled',
    'CompletedAlready'
);

CREATE TABLE IF NOT EXISTS public.analysis_queue (
    game_id CHAR(8) NOT NULL,
    requesting_user_id CHAR(10),
    requested_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    started_unix_sec BIGINT,
    status analysis_queue_status NOT NULL DEFAULT 'Pending',
    FOREIGN KEY (requesting_user_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
);

-- When a user logs in, they get a session token.
CREATE TABLE IF NOT EXISTS social.user_sessions (
    session_id SERIAL PRIMARY KEY,
    created_at_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    expires_at_unix_sec BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) + 3600 * 24), -- 1 day
    user_id CHAR(10) NOT NULL REFERENCES social.registered_users(id) ON DELETE CASCADE,
    session_token_hash TEXT NOT NULL UNIQUE
);
