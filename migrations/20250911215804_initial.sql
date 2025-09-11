-- Create playlists table
CREATE TABLE IF NOT EXISTS playlists (
    id BLOB PRIMARY KEY,
    spotify_id TEXT,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

-- Create tracks table
CREATE TABLE IF NOT EXISTS tracks (
    id BLOB PRIMARY KEY,
    playlist_id BLOB NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    year INTEGER NOT NULL,
    spotify_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
);

-- Create jobs table
CREATE TABLE IF NOT EXISTS jobs (
    id BLOB PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    kind TEXT NOT NULL CHECK (kind IN ('generate_pdfs', 'refetch_playlist')),
    payload TEXT NOT NULL DEFAULT '{}'
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_playlists_spotify_id ON playlists(spotify_id);
CREATE INDEX IF NOT EXISTS idx_playlists_created_at ON playlists(created_at);
CREATE INDEX IF NOT EXISTS idx_tracks_playlist_id ON tracks(playlist_id);
CREATE INDEX IF NOT EXISTS idx_tracks_position ON tracks(position);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_kind ON jobs(kind);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_jobs_payload_playlist_id ON jobs(json_extract(payload, '$.playlist_id'));

-- Create triggers for updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_playlists_updated_at 
    AFTER UPDATE ON playlists
    FOR EACH ROW
BEGIN
    UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;