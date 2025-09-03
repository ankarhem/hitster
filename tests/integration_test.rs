//! Integration tests for Hitster
//! 
//! This module contains integration tests that verify the entire workflow
//! from configuration loading to HTML generation.

use hitster::{Settings, SpotifyService, HtmlGenerator, PlaylistId};

#[tokio::test]
async fn test_full_workflow_with_valid_config() {
    // Skip test if no valid credentials are available
    let settings = match Settings::new() {
        Ok(settings) if settings.is_valid() => settings,
        _ => {
            println!("Skipping test - no valid Spotify credentials available");
            return;
        }
    };

    // Test Spotify service creation
    let spotify_service = match SpotifyService::new(&settings).await {
        Ok(service) => service,
        Err(e) => {
            println!("Skipping test - Spotify authentication failed: {}", e);
            return;
        }
    };

    // Test with a known valid playlist ID
    let playlist_id: PlaylistId = "3vnwX8FuGWpGgQX4hBa8sE".parse().unwrap();
    
    // Test playlist fetching
    let playlist = match spotify_service.get_playlist(playlist_id).await {
        Ok(playlist) => playlist,
        Err(e) => {
            println!("Skipping test - Playlist fetch failed: {}", e);
            return;
        }
    };
    let cards = playlist.tracks;

    // Verify we got some cards
    assert!(!cards.is_empty(), "Playlist should contain at least one track");

    // Test HTML generation
    let html_generator = HtmlGenerator::new().unwrap();
    let html_content = html_generator.build_html_content(cards.clone(), "Test Playlist").unwrap();
    
    // Verify HTML content
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("Test Playlist"));
    assert!(html_content.contains("songs"));
    
    // Verify dual-sided layout
    assert!(html_content.contains("Front Pages (QR Codes)"));
    assert!(html_content.contains("hidden print:block"));
    assert!(html_content.contains("print:break-after-page"));
    assert!(html_content.contains("grid-template-columns: repeat(3, 70mm)"));
    assert!(html_content.contains("grid-template-rows: repeat(4, 70mm)"));
    
    // Verify the first song is present
    if let Some(first_card) = cards.first() {
        assert!(html_content.contains(&first_card.title));
        assert!(html_content.contains(&first_card.artist));
    }
}

#[test]
fn test_configuration_validation() {
    // Test with empty credentials
    let settings = Settings {
        client_id: String::new(),
        client_secret: String::new(),
    };
    assert!(!settings.is_valid());

    // Test with partial credentials
    let settings = Settings {
        client_id: "test_id".to_string(),
        client_secret: String::new(),
    };
    assert!(!settings.is_valid());

    // Test with valid credentials
    let settings = Settings {
        client_id: "test_id".to_string(),
        client_secret: "test_secret".to_string(),
    };
    assert!(settings.is_valid());
}

#[test]
fn test_playlist_id_parsing() {
    // Test direct ID
    let id: PlaylistId = "3vnwX8FuGWpGgQX4hBa8sE".parse().unwrap();
    assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");

    // Test URL parsing
    let url = "https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE";
    let id: PlaylistId = url.parse().unwrap();
    assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");

    // Test URL with query parameters
    let url_with_query = "https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE?si=abc123";
    let id: PlaylistId = url_with_query.parse().unwrap();
    assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
}

#[tokio::test]
async fn test_html_generator_basic() {
    let html_generator = HtmlGenerator::new().unwrap();
    
    // Create test cards
    let cards = vec![
        hitster::Track {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            year: "2023".to_string(),
            spotify_url: "https://open.spotify.com/track/test".to_string(),
        },
    ];
    
    // Test HTML content generation
    let html = html_generator.build_html_content(cards, "Test Playlist").unwrap();
    
    // Verify basic structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<body"));
    assert!(html.contains("Test Playlist"));
    assert!(html.contains("Test Song"));
    assert!(html.contains("Test Artist"));
    assert!(html.contains("2023"));
    
    // Verify dual-sided layout and print optimization
    assert!(html.contains("@page"));
    assert!(html.contains("tailwindcss"));
    assert!(html.contains("Front Pages (QR Codes)"));
    assert!(html.contains("hidden print:block"));
    assert!(html.contains("print:break-after-page"));
    assert!(html.contains("width: 60mm"));
    assert!(html.contains("font-size: 24px"));
}

#[tokio::test]
async fn test_error_handling_scenarios() {
    // Test invalid playlist ID
    let invalid_id = "";
    let result: Result<PlaylistId, _> = invalid_id.parse();
    assert!(result.is_err());

    // Test malformed URL
    let malformed_url = "https://open.spotify.com/playlist/";
    let result: Result<PlaylistId, _> = malformed_url.parse();
    assert!(result.is_err());

    // Test configuration validation with empty values
    let settings = Settings {
        client_id: String::new(),
        client_secret: String::new(),
    };
    assert!(!settings.is_valid());
}