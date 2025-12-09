-- Down migration for initial schema: drop created objects in reverse order

-- Drop lobby path generation trigger and function
DROP TRIGGER IF EXISTS trigger_set_lobby_path ON lobbies;
DROP FUNCTION IF EXISTS set_lobby_path();
DROP FUNCTION IF EXISTS generate_lobby_path();

DROP TABLE IF EXISTS platform_ratings;
DROP TABLE IF EXISTS user_wars_points;
DROP TABLE IF EXISTS lobbies;
DROP TABLE IF EXISTS games;
DROP TABLE IF EXISTS seasons;
DROP TABLE IF EXISTS users;

DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'lobby_status') THEN
        DROP TYPE lobby_status;
    END IF;
END$$;
