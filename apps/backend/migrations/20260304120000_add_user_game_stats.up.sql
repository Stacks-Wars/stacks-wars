-- Add per-game leaderboard stats table

CREATE TABLE user_game_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    season_id INT NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    points DOUBLE PRECISION DEFAULT 0,
    total_matches INTEGER DEFAULT 0,
    total_wins INTEGER DEFAULT 0,
    total_pnl DOUBLE PRECISION DEFAULT 0.0,
    win_rate DOUBLE PRECISION DEFAULT 0.0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, game_id, season_id)
);

CREATE INDEX idx_user_game_stats_game_season ON user_game_stats(game_id, season_id);
CREATE INDEX idx_user_game_stats_user ON user_game_stats(user_id);
CREATE INDEX idx_user_game_stats_points ON user_game_stats(game_id, season_id, points DESC);
