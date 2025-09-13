-- Add album_cover_url column to tracks table
ALTER TABLE tracks ADD COLUMN album_cover_url TEXT;

-- Add index for better performance on album cover queries
CREATE INDEX IF NOT EXISTS idx_tracks_album_cover_url ON tracks(album_cover_url);
