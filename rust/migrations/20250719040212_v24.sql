-- Add migration script here
-- Alter lobbies: Add new column playerTeams
CREATE TABLE analysis_1.spawn_locations (
    game_id CHAR(8) NOT NULL,
    client_id CHAR(8) NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    previous_spawns JSONB DEFAULT '[]',
    PRIMARY KEY (game_id, client_id),
    FOREIGN KEY (game_id) REFERENCES public.lobbies(game_id) ON DELETE CASCADE
);

CREATE SCHEMA social;


-- Create a function to generate a random user ID
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

CREATE TABLE social.registered_users (
    id CHAR(10) NOT NULL PRIMARY KEY DEFAULT social.generate_user_uid(10),
    username TEXT NOT NULL UNIQUE,
    openfront_player_id TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE social.friends (
    user_id CHAR(10) NOT NULL,
    friend_id CHAR(10) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, friend_id),
    FOREIGN KEY (user_id) REFERENCES social.registered_users(id) ON DELETE CASCADE,
    FOREIGN KEY (friend_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
);

-- "Alliances" in webapp
CREATE TABLE social.friend_requests (
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
    'Failed',
    'Stalled',
    'Cancelled',
    'CompletedAlready'
);

CREATE TABLE public.analysis_queue (
    game_id CHAR(8) NOT NULL,
    requesting_user_id CHAR(10),
    requested_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    started_unix_sec BIGINT,
    status analysis_queue_status NOT NULL DEFAULT 'Pending',
    FOREIGN KEY (requesting_user_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
);
