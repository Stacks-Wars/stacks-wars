-- Add player stats to user_wars_points, add profile_image to users, change games.category to array

-- Add profile_image to users table
ALTER TABLE users ADD COLUMN profile_image TEXT;

-- Change games.category from TEXT to TEXT[] (array) - handle existing data
ALTER TABLE games ALTER COLUMN category TYPE TEXT[] USING ARRAY[category];

-- Add player statistics to user_wars_points table
ALTER TABLE user_wars_points ADD COLUMN total_matches INTEGER DEFAULT 0;
ALTER TABLE user_wars_points ADD COLUMN total_wins INTEGER DEFAULT 0;
ALTER TABLE user_wars_points ADD COLUMN total_pnl DOUBLE PRECISION DEFAULT 0.0;
ALTER TABLE user_wars_points ADD COLUMN win_rate DOUBLE PRECISION DEFAULT 0.0;