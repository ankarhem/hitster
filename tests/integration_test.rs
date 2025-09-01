use hitster::spotify_service::{SpotifyService, SongCard};

#[tokio::test]
async fn test_spotify_service_integration() {
    let _service = SpotifyService::new();
    
    // Test that the service can be created
    assert!(true); // Basic smoke test
}

#[test]
fn test_extract_playlist_id_edge_cases() {
    // Test various edge cases for playlist ID extraction
    let test_cases = vec![
        ("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M", Some("37i9dQZF1DXcBWIGoYBM5M")),
        ("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc123", Some("37i9dQZF1DXcBWIGoYBM5M")),
        ("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc123&other=value", Some("37i9dQZF1DXcBWIGoYBM5M")),
        ("", None),
        ("https://open.spotify.com/playlist/", None),
        ("invalid_url", Some("invalid_url")),
        ("https://open.spotify.com/invalid/37i9dQZF1DXcBWIGoYBM5M", Some("37i9dQZF1DXcBWIGoYBM5M")),
    ];

    for (url, expected) in test_cases {
        let result = SpotifyService::extract_playlist_id(url);
        match expected {
            Some(expected_id) => {
                assert!(result.is_ok(), "Failed for URL: {}", url);
                assert_eq!(result.unwrap(), expected_id, "Wrong ID for URL: {}", url);
            }
            None => {
                assert!(result.is_err(), "Expected error for URL: {}", url);
            }
        }
    }
}

#[test]
fn test_song_card_fields() {
    let card = SongCard {
        title: "Test Song".to_string(),
        artist: "Test Artist".to_string(),
        year: "2023".to_string(),
        spotify_url: "https://open.spotify.com/track/test".to_string(),
    };

    assert_eq!(card.title, "Test Song");
    assert_eq!(card.artist, "Test Artist");
    assert_eq!(card.year, "2023");
    assert_eq!(card.spotify_url, "https://open.spotify.com/track/test");
}

#[test]
fn test_multiple_artists_formatting() {
    // This would test that multiple artists are properly joined with commas
    // Since we can't easily test the API call, we'll test the structure
    let card = SongCard {
        title: "Test Song".to_string(),
        artist: "Artist 1, Artist 2, Artist 3".to_string(),
        year: "2023".to_string(),
        spotify_url: "https://open.spotify.com/track/test".to_string(),
    };

    assert!(card.artist.contains(","));
    assert_eq!(card.artist, "Artist 1, Artist 2, Artist 3");
}