-- ===============================
-- 🚀 STACKS WARS INITIAL SCHEMA
-- ===============================
-- This migration creates all core tables and relations in one pass.

-- Enable UUID extension (for gen_random_uuid)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ======================================
-- USERS TABLE
-- ======================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT UNIQUE NOT NULL,
    username CITEXT UNIQUE, -- case-insensitive + unique
    display_name TEXT,
    trust_rating DOUBLE PRECISION DEFAULT 10,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Add CITEXT extension for case-insensitive username
CREATE EXTENSION IF NOT EXISTS citext;

-- Indexing for faster lookup
CREATE INDEX idx_users_wallet_address ON users(wallet_address);
CREATE INDEX idx_users_username ON users(username);

-- ======================================
-- SEASONS TABLE
-- ======================================
CREATE TABLE seasons (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- ======================================
-- GAMES TABLE
-- ======================================
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

CREATE INDEX idx_games_creator_id ON games(creator_id);
CREATE INDEX idx_games_is_active ON games(is_active);

-- ======================================
-- ENUM TYPE: LOBBY STATUS
-- ======================================
CREATE TYPE lobby_status AS ENUM ('waiting', 'starting', 'in_progress', 'finished');

-- ======================================
-- LOBBIES TABLE
-- ======================================
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

CREATE INDEX idx_lobbies_game_id ON lobbies(game_id);
CREATE INDEX idx_lobbies_creator_id ON lobbies(creator_id);
CREATE INDEX idx_lobbies_status ON lobbies(status);
CREATE INDEX idx_lobbies_is_sponsored ON lobbies(is_sponsored);

-- ======================================
-- USER WARS POINTS TABLE
-- ======================================
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

CREATE INDEX idx_user_wars_points_user_id ON user_wars_points(user_id);
CREATE INDEX idx_user_wars_points_season_id ON user_wars_points(season_id);

-- ======================================
-- PLATFORM RATINGS TABLE
-- ======================================
CREATE TABLE platform_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    rating SMALLINT CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE INDEX idx_platform_ratings_user_id ON platform_ratings(user_id);
CREATE INDEX idx_platform_ratings_rating ON platform_ratings(rating);

-- ======================================
-- ✅ COMPLETION LOG
-- ======================================
COMMENT ON DATABASE current_database() IS 'Stacks Wars initial database schema (users, games, lobbies, seasons, points, ratings).';
