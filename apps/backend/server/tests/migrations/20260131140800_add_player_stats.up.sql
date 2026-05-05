-- Add player stats to user_wars_points, add profile_image to users, change games.category to array

-- Add profile_image to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS profile_image TEXT;

-- Change games.category from TEXT to TEXT[] (array) - handle existing data
DO $$ BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'games' AND column_name = 'category' AND data_type = 'text'
    ) THEN
        ALTER TABLE games ALTER COLUMN category TYPE TEXT[] USING ARRAY[category];
    END IF;
END $$;

-- Add player statistics to user_wars_points table
ALTER TABLE user_wars_points ADD COLUMN IF NOT EXISTS total_matches INTEGER DEFAULT 0;
ALTER TABLE user_wars_points ADD COLUMN IF NOT EXISTS total_wins INTEGER DEFAULT 0;
ALTER TABLE user_wars_points ADD COLUMN IF NOT EXISTS total_pnl DOUBLE PRECISION DEFAULT 0.0;
ALTER TABLE user_wars_points ADD COLUMN IF NOT EXISTS win_rate DOUBLE PRECISION DEFAULT 0.0;
