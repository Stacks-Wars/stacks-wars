-- Revert: Remove player stats from user_wars_points, remove profile_image from users, change games.category back to TEXT

-- Remove player statistics from user_wars_points table
ALTER TABLE user_wars_points DROP COLUMN win_rate;
ALTER TABLE user_wars_points DROP COLUMN total_pnl;
ALTER TABLE user_wars_points DROP COLUMN total_wins;
ALTER TABLE user_wars_points DROP COLUMN total_matches;

-- Change games.category back from TEXT[] to TEXT
ALTER TABLE games ALTER COLUMN category TYPE TEXT;

-- Remove profile_image from users table
ALTER TABLE users DROP COLUMN profile_image;