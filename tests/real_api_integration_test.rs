use anyhow::Result;
use hitster::{SpotifyService, Settings};

const TEST_PLAYLIST_URL: &str = "https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE?si=1d7567f5e8bd4c95";

#[tokio::test]
async fn test_real_spotify_api_integration() -> Result<()> {
    let settings = Settings::new()?;
    let spotify_service = SpotifyService::new(&settings).await?;
    let cards = spotify_service.get_playlist_tracks(TEST_PLAYLIST_URL).await?;

    // Verify we got some tracks
    assert!(!cards.is_empty(), "Playlist should return at least one track");
    println!("Found {} tracks in playlist", cards.len());

    // Verify specific songs are present
    let expected_songs = vec![
        ("Stressed Out", "Twenty One Pilots"),
        ("Lane Boy", "Twenty One Pilots"),
        ("Shake That", "Eminem"),
    ];

    for (title, artist) in expected_songs {
        let found = cards.iter().any(|card| {
            card.title == title && card.artist.contains(artist)
        });
        
        assert!(found, "Expected song '{}' by '{}' not found in playlist", title, artist);
        println!("✓ Found '{}' by '{}'", title, artist);
    }

    // Print all found tracks for debugging
    println!("\nAll tracks found in playlist:");
    for (i, card) in cards.iter().enumerate() {
        println!("{}. {} - {} ({})", i + 1, card.title, card.artist, card.year);
    }

    // Verify that each card has required fields
    for card in &cards {
        assert!(!card.title.is_empty(), "Song title should not be empty");
        assert!(!card.artist.is_empty(), "Artist should not be empty");
        assert!(!card.year.is_empty(), "Year should not be empty");
        assert!(!card.spotify_url.is_empty(), "Spotify URL should not be empty");
        assert!(card.spotify_url.starts_with("https://open.spotify.com/"), "Spotify URL should be valid");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_real_spotify_api_invalid_credentials() {
    // Test with invalid credentials to ensure proper error handling
    let invalid_settings = Settings {
        spotify: hitster::config::SpotifySettings {
            client_id: "invalid_client_id".to_string(),
            client_secret: "invalid_client_secret".to_string(),
        },
    };

    let result = SpotifyService::new(&invalid_settings).await;
    assert!(result.is_err(), "Should fail with invalid credentials");
    
    match result.err().unwrap() {
        _ => println!("✓ Correctly failed with invalid credentials"),
    }
}

#[tokio::test]
async fn test_real_spotify_api_invalid_playlist() -> Result<()> {
    let settings = Settings::new()?;
    let spotify_service = SpotifyService::new(&settings).await?;
    
    // Test with invalid playlist URL
    let invalid_playlist_url = "https://open.spotify.com/playlist/invalid_playlist_id";
    let result = spotify_service.get_playlist_tracks(invalid_playlist_url).await;
    assert!(result.is_err(), "Should fail with invalid playlist URL");
    
    Ok(())
}

#[tokio::test]
async fn test_extract_playlist_id_from_real_url() -> Result<()> {
    let playlist_id = hitster::SpotifyService::extract_playlist_id(TEST_PLAYLIST_URL)?;
    assert_eq!(playlist_id, "3vnwX8FuGWpGgQX4hBa8sE", "Should extract correct playlist ID");
    Ok(())
}