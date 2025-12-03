-- Stacks Wars initial schema (sqlx format)

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS citext;

-- USERS
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT UNIQUE NOT NULL,
    username CITEXT UNIQUE,
    display_name TEXT,
    trust_rating DOUBLE PRECISION DEFAULT 10,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(wallet_address);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- SEASONS
CREATE TABLE seasons (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- GAMES
CREATE TABLE games (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    image_url TEXT NOT NULL,
    min_players SMALLINT NOT NULL CHECK (min_players >= 1 AND min_players <= 255),
    max_players SMALLINT NOT NULL CHECK (max_players >= 1 AND max_players <= 255),
    category TEXT,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_games_creator_id ON games(creator_id);
CREATE INDEX IF NOT EXISTS idx_games_is_active ON games(is_active);

-- ENUM TYPE: LOBBY STATUS
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'lobby_status') THEN
        CREATE TYPE lobby_status AS ENUM ('waiting', 'starting', 'in_progress', 'finished');
    END IF;
END$$;

-- LOBBIES
CREATE TABLE lobbies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entry_amount DOUBLE PRECISION,
    current_amount DOUBLE PRECISION DEFAULT 0,
    token_symbol TEXT,
    token_contract_id TEXT,
    contract_address TEXT,
    is_private BOOLEAN DEFAULT FALSE,
    is_sponsored BOOLEAN DEFAULT FALSE,
    status lobby_status DEFAULT 'waiting',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_lobbies_game_id ON lobbies(game_id);
CREATE INDEX IF NOT EXISTS idx_lobbies_creator_id ON lobbies(creator_id);
CREATE INDEX IF NOT EXISTS idx_lobbies_status ON lobbies(status);
CREATE INDEX IF NOT EXISTS idx_lobbies_is_sponsored ON lobbies(is_sponsored);

-- USER WARS POINTS
CREATE TABLE user_wars_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    season_id INT REFERENCES seasons(id) ON DELETE CASCADE,
    points DOUBLE PRECISION DEFAULT 0,
    rank_badge TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, season_id)
);

CREATE INDEX IF NOT EXISTS idx_user_wars_points_user_id ON user_wars_points(user_id);
CREATE INDEX IF NOT EXISTS idx_user_wars_points_season_id ON user_wars_points(season_id);

-- PLATFORM RATINGS
CREATE TABLE platform_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    rating SMALLINT CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE INDEX IF NOT EXISTS idx_platform_ratings_user_id ON platform_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_platform_ratings_rating ON platform_ratings(rating);

COMMENT ON DATABASE postgres IS 'Stacks Wars initial database schema (users, games, lobbies, seasons, points, ratings).';
