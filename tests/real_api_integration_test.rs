use anyhow::Result;
use hitster::{SpotifyService, Settings, PlaylistId};

const TEST_PLAYLIST_URL: &str = "https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE?si=1d7567f5e8bd4c95";

#[tokio::test]
async fn test_real_spotify_api_invalid_credentials() {
    // Test with invalid credentials to ensure proper error handling
    let invalid_settings = Settings {
        client_id: "invalid_client_id".to_string(),
        client_secret: "invalid_client_secret".to_string(),
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
    let invalid_playlist_id: PlaylistId = invalid_playlist_url.parse()?;
    let result = spotify_service.get_playlist(invalid_playlist_id).await;
    assert!(result.is_err(), "Should fail with invalid playlist URL");
    
    Ok(())
}

#[tokio::test]
async fn test_extract_playlist_id_from_real_url() -> Result<()> {
    let playlist_id: hitster::PlaylistId = TEST_PLAYLIST_URL.parse()?;
    assert_eq!(playlist_id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE", "Should extract correct playlist ID");
    Ok(())
}

#[tokio::test]
async fn test_real_spotify_api_integration() -> Result<()> {
    let settings = Settings::new()?;
    let spotify_service = SpotifyService::new(&settings).await?;
    let playlist_id: PlaylistId = TEST_PLAYLIST_URL.parse()?;
    let playlist = spotify_service.get_playlist(playlist_id).await?;
    let cards = playlist.tracks;

    // Assert exact output for all songs
    let expected_cards = vec![
        hitster::Track {
            title: "Stressed Out".to_string(),
            artist: "Twenty One Pilots".to_string(),
            year: "2015".to_string(),
            spotify_url: "https://open.spotify.com/track/3CRDbSIZ4r5MsZ0YwxuEkn".to_string(),
        },
        hitster::Track {
            title: "Lane Boy".to_string(),
            artist: "Twenty One Pilots".to_string(),
            year: "2015".to_string(),
            spotify_url: "https://open.spotify.com/track/2P61EK6DMGyVyssLWS4fKy".to_string(),
        },
        hitster::Track {
            title: "Shake That".to_string(),
            artist: "Eminem, Nate Dogg".to_string(),
            year: "2005".to_string(),
            spotify_url: "https://open.spotify.com/track/6KqKg8IPuvtDB3PNAvffFf".to_string(),
        },
    ];

    assert_eq!(cards, expected_cards, "Playlist should return exactly the expected songs");

    Ok(())
}
