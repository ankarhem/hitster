use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;
use winnow::combinator::preceded;
use winnow::token::rest;
use winnow::{Parser, combinator::alt, token::take_while};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpotifyId(String);

impl SpotifyId {
    /// Parse a Spotify ID from various formats:
    /// - URL: http://open.spotify.com/playlist/6rqhFgbbKwnb9MLmUQDhG6
    /// - URI: spotify:playlist:6rqhFgbbKwnb9MLmUQDhG6
    /// - Raw: 6rqhFgbbKwnb9MLmUQDhG6
    pub fn parse(input: &str) -> Result<Self, SpotifyIdParserError> {
        let id = spotify_id_parser
            .parse(input)
            .map_err(|_| SpotifyIdParserError::InvalidFormat(input.to_string()))?;
        Ok(Self(id))
    }

    /// Get the raw Spotify ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the Spotify URL for this ID
    pub fn as_url(&self) -> String {
        format!("https://open.spotify.com/playlist/{}", self.0)
    }

    /// Get the Spotify URI for this ID
    pub fn as_uri(&self) -> String {
        format!("spotify:playlist:{}", self.0)
    }
}

impl std::fmt::Display for SpotifyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SpotifyId {
    type Err = SpotifyIdParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<SpotifyId> for String {
    fn from(id: SpotifyId) -> Self {
        id.0
    }
}

/// Custom error type for Spotify ID parsing
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpotifyIdParserError {
    InvalidFormat(String),
}

impl std::fmt::Display for SpotifyIdParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpotifyIdParserError::InvalidFormat(input) => {
                write!(f, "Invalid Spotify ID format: {}", input)
            }
        }
    }
}

/// Winnow parser for Spotify ID formats
fn spotify_id_parser(input: &mut &str) -> winnow::Result<String> {
    alt((parse_url_format, parse_uri_format, parse_raw_id)).parse_next(input)
}

/// Parse URL format: http://open.spotify.com/playlist/6rqhFgbbKwnb9MLmUQDhG6
fn parse_url_format(input: &mut &str) -> winnow::Result<String> {
    let base_url = preceded(alt(("http://", "https://")), "open.spotify.com/playlist/");
    let id = preceded(base_url, parse_raw_id).parse_next(input)?;

    // Consume any trailing query parameters
    let _ = rest.parse_next(input)?;

    Ok(id)
}

/// Parse URI format: spotify:playlist:6rqhFgbbKwnb9MLmUQDhG6
fn parse_uri_format(input: &mut &str) -> winnow::Result<String> {
    preceded("spotify:playlist:", parse_raw_id).parse_next(input)
}

/// Parse raw ID format: 6rqhFgbbKwnb9MLmUQDhG6
fn parse_raw_id(input: &mut &str) -> winnow::Result<String> {
    take_while(1.., |c: char| c.is_alphanumeric())
        .verify(|id: &str| !id.is_empty())
        .map(|id: &str| id.to_string())
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_formats() {
        assert_eq!(
            &SpotifyId::parse("6rqhFgbbKwnb9MLmUQDhG6")
                .unwrap()
                .to_string(),
            "6rqhFgbbKwnb9MLmUQDhG6"
        );
        assert_eq!(
            &SpotifyId::parse("spotify:playlist:6rqhFgbbKwnb9MLmUQDhG6")
                .unwrap()
                .to_string(),
            "6rqhFgbbKwnb9MLmUQDhG6"
        );
        assert_eq!(
            &SpotifyId::parse("http://open.spotify.com/playlist/6rqhFgbbKwnb9MLmUQDhG6")
                .unwrap()
                .to_string(),
            "6rqhFgbbKwnb9MLmUQDhG6"
        );
        assert_eq!(
            &SpotifyId::parse(
                "https://open.spotify.com/playlist/6rqhFgbbKwnb9MLmUQDhG6?si=403cff98ac8d4479"
            )
            .unwrap()
            .to_string(),
            "6rqhFgbbKwnb9MLmUQDhG6"
        );
    }

    #[test]
    fn test_invalid_formats() {
        assert!(SpotifyId::parse("").is_err());
        assert!(SpotifyId::parse("invalid-url").is_err());
        assert!(SpotifyId::parse("http://invalid.com/playlist/abc").is_err());
        assert!(SpotifyId::parse("spotify:invalid:abc").is_err());
    }
}
