#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use tokio::runtime::Runtime;
    use crate::infrastructure::database::{Database, JobStatus, NewTrack};

    #[test]
    fn test_database_initialization() {
        // Create a temporary database file
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        // Test that we can create a database
        let result = Runtime::new().unwrap().block_on(async {
            Database::new(db_path).await
        });
        
        assert!(result.is_ok(), "Failed to initialize database: {:?}", result.err());
        
        // Test that we can create a playlist
        let database = result.unwrap();
        let playlist_result = Runtime::new().unwrap().block_on(async {
            database.create_playlist("test_playlist", "Test Playlist").await
        });
        
        assert!(playlist_result.is_ok(), "Failed to create playlist: {:?}", playlist_result.err());
        
        let playlist = playlist_result.unwrap();
        assert_eq!(playlist.spotify_id, "test_playlist");
        assert_eq!(playlist.name, "Test Playlist");
        assert!(playlist.id > 0);
    }

    #[test]
    fn test_job_creation_and_status() {
        // Create a temporary database file
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let database = Runtime::new().unwrap().block_on(async {
            Database::new(db_path).await.unwrap()
        });
        
        // Create a playlist first
        let playlist = Runtime::new().unwrap().block_on(async {
            database.create_playlist("test_playlist", "Test Playlist").await
        }).unwrap();
        
        // Test job creation
        let job = Runtime::new().unwrap().block_on(async {
            database.create_job(playlist.id).await
        }).unwrap();
        
        assert_eq!(job.playlist_id, playlist.id);
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.id > 0);
        
        // Test job status update
        let update_result = Runtime::new().unwrap().block_on(async {
            database.update_job_status(job.id, JobStatus::Processing).await
        });
        
        assert!(update_result.is_ok(), "Failed to update job status: {:?}", update_result.err());
        
        // Verify the status was updated
        let updated_job = Runtime::new().unwrap().block_on(async {
            database.get_job_by_id(job.id).await
        }).unwrap().unwrap();
        
        assert_eq!(updated_job.status, JobStatus::Processing);
    }

    #[test]
    fn test_track_creation_and_retrieval() {
        // Create a temporary database file
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        
        let database = Runtime::new().unwrap().block_on(async {
            Database::new(db_path).await.unwrap()
        });
        
        // Create a playlist
        let playlist = Runtime::new().unwrap().block_on(async {
            database.create_playlist("test_playlist", "Test Playlist").await
        }).unwrap();
        
        // Create test tracks
        let tracks = vec![
            NewTrack {
                playlist_id: playlist.id,
                title: "Test Song 1".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://spotify.com/test1".to_string(),
                position: 1,
            },
            NewTrack {
                playlist_id: playlist.id,
                title: "Test Song 2".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://spotify.com/test2".to_string(),
                position: 2,
            },
        ];
        
        // Create tracks
        let create_result = Runtime::new().unwrap().block_on(async {
            database.create_tracks(&tracks).await
        });
        
        assert!(create_result.is_ok(), "Failed to create tracks: {:?}", create_result.err());
        
        // Retrieve tracks
        let retrieved_tracks = Runtime::new().unwrap().block_on(async {
            database.get_tracks_by_playlist_id(playlist.id).await
        }).unwrap();
        
        assert_eq!(retrieved_tracks.len(), 2);
        assert_eq!(retrieved_tracks[0].title, "Test Song 1");
        assert_eq!(retrieved_tracks[1].title, "Test Song 2");
        assert_eq!(retrieved_tracks[0].position, 1);
        assert_eq!(retrieved_tracks[1].position, 2);
    }

    #[test]
    fn test_spotify_playlist_url_parsing() {
        use crate::web::server::extract_playlist_id;
        
        // Test various URL formats
        let test_cases = vec![
            ("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M", "37i9dQZF1DXcBWIGoYBM5M"),
            ("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M", "37i9dQZF1DXcBWIGoYBM5M"),
            ("37i9dQZF1DXcBWIGoYBM5M", "37i9dQZF1DXcBWIGoYBM5M"),
        ];
        
        for (input, expected) in test_cases {
            let result = extract_playlist_id(input);
            assert!(result.is_ok(), "Failed to parse URL: {}", input);
            assert_eq!(result.unwrap(), expected);
        }
    }
}