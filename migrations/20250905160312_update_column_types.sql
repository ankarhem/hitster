-- Update column types to use more specific types

-- Note: SQLite doesn't support direct column type changes for existing data
-- We need to create new tables with the correct types and migrate the data

-- Disable foreign key constraints temporarily
PRAGMA foreign_keys = OFF;

-- Create new playlists table with correct types
CREATE TABLE IF NOT EXISTS playlists_new (
    id BLOB PRIMARY KEY,
    spotify_id TEXT,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

-- Create new jobs table with correct types
CREATE TABLE IF NOT EXISTS jobs_new (
    id BLOB PRIMARY KEY,
    playlist_id BLOB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    front_pdf_path TEXT,
    back_pdf_path TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

-- Create new tracks table with correct types
CREATE TABLE IF NOT EXISTS tracks_new (
    id BLOB PRIMARY KEY,
    playlist_id BLOB NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    year INTEGER NOT NULL,
    spotify_url TEXT NOT NULL,
    position INTEGER NOT NULL
);

-- Migrate data from old tables to new tables
-- Playlists
INSERT INTO playlists_new (id, spotify_id, name, created_at, updated_at)
SELECT 
    CAST(id AS BLOB),
    spotify_id,
    name,
    created_at,
    updated_at
FROM playlists;

-- Jobs
INSERT INTO jobs_new (id, playlist_id, status, front_pdf_path, back_pdf_path, created_at, completed_at)
SELECT 
    CAST(id AS BLOB),
    CAST(playlist_id AS BLOB),
    status,
    front_pdf_path,
    back_pdf_path,
    created_at,
    completed_at
FROM jobs;

-- Tracks
INSERT INTO tracks_new (id, playlist_id, title, artist, year, spotify_url, position)
SELECT 
    CAST(id AS BLOB),
    CAST(playlist_id AS BLOB),
    title,
    artist,
    CAST(year AS INTEGER),
    spotify_url,
    position
FROM tracks;

-- Drop old tables
DROP TABLE playlists;
DROP TABLE jobs;
DROP TABLE tracks;

-- Rename new tables to original names
ALTER TABLE playlists_new RENAME TO playlists;
ALTER TABLE jobs_new RENAME TO jobs;
ALTER TABLE tracks_new RENAME TO tracks;

-- Recreate indexes if needed
CREATE INDEX IF NOT EXISTS idx_playlists_spotify_id ON playlists(spotify_id);
CREATE INDEX IF NOT EXISTS idx_jobs_playlist_id ON jobs(playlist_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_tracks_playlist_id ON tracks(playlist_id);
CREATE INDEX IF NOT EXISTS idx_tracks_position ON tracks(position);

-- Re-enable foreign key constraints
PRAGMA foreign_keys = ON;