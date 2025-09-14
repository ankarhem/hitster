use crate::Settings;
use crate::application::ISpotifyClient;
use crate::domain;
use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt};
use rspotify::model::{PlayableItem};
use rspotify::{ClientCredsSpotify, Credentials, prelude::BaseClient};
use tracing::{error, info, instrument};

#[derive(Clone)]
pub struct SpotifyClient {
    client: ClientCredsSpotify,
}

impl SpotifyClient {
    #[instrument(skip(settings))]
    pub async fn new(settings: &Settings) -> Result<Self> {
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);
        let client = ClientCredsSpotify::new(creds);
        client.request_token().await?;
        info!("Spotify authentication successful");

        Ok(Self { client })
    }
}

impl ISpotifyClient for SpotifyClient {
    #[instrument(skip(self), fields(id = %id))]
    async fn get_playlist_with_tracks(&self, id: &domain::SpotifyId) -> Result<Option<domain::Playlist>> {
        let spotify_id = id.to_string();
        let rspotify_playlist_id = rspotify::model::PlaylistId::from_id_or_uri(&spotify_id)?;

        let full_playlist = self
            .client
            .playlist(rspotify_playlist_id, None, None)
            .await?;
        
        let limit = full_playlist.tracks.limit;

        // The first request includes the first 100 tracks
        // we can create a stream to push them into and then fetch the rest
        let first_100_tracks = full_playlist.tracks.items;

        // this will round down, which is what we want (because we already have the first page)
        let pages_to_fetch = &full_playlist.tracks.total / limit;
        let futures = (0..pages_to_fetch).map(|page| {
            let offset = 100 + page * limit;
            let client = &self.client;
            let playlist_id = full_playlist.id.clone();
            async move {
                client
                    .playlist_items_manual(playlist_id, None, None, Some(limit), Some(offset))
                    .await
            }
        });

        let first_page_stream = futures_util::stream::iter(first_100_tracks);
        let tracks_stream = futures_util::stream::iter(futures)
            .buffer_unordered(5)
            .map(|res| match res {
                Ok(page) => page.items,
                Err(e) => {
                    // Log the error and return an empty vector for this page
                    // In a real application, you might want to handle this differently
                    error!("Error fetching playlist page: {}", e);
                    Vec::new()
                }
            })
            .flat_map(futures_util::stream::iter);
        // Create a stream of all tracks by combining the first 100 tracks with the rest
        let full_stream = first_page_stream.chain(tracks_stream);

        let tracks = full_stream
            .map(|item| async move {
                match item.track {
                    Some(PlayableItem::Track(track)) => {
                        let domain_track = track.try_into().ok();
                        Ok::<Option<domain::Track>, anyhow::Error>(domain_track)
                    },
                    _ => {
                        // Skip non-track items
                        Ok(None)
                    },
                }
            })
            .buffer_unordered(50)
            .try_collect::<Vec<Option<domain::Track>>>()
            .await?;

        let tracks: Vec<domain::Track> = tracks.into_iter().flatten().collect();

        Ok(Some(domain::Playlist {
            id: domain::PlaylistId::new()?,
            name: full_playlist.name,
            tracks,
            spotify_id: Some(id.clone()),
            created_at: None,
            updated_at: None,
        }))
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn get_playlist(&self, id: &domain::SpotifyId) -> Result<Option<domain::Playlist>> {
        let spotify_id = id.to_string();
        let rspotify_playlist_id = rspotify::model::PlaylistId::from_id_or_uri(&spotify_id)?;
        let full_playlist = self
            .client
            .playlist(rspotify_playlist_id, None, None)
            .await?;

        Ok(Some(domain::Playlist {
            id: domain::PlaylistId::new()?,
            name: full_playlist.name,
            tracks: Vec::new(),
            spotify_id: Some(id.clone()),
            created_at: None,
            updated_at: None,
        }))
    }
}

mod conversions {
    use crate::domain::Track;
    use anyhow::{Result, bail, Context};
    use chrono::{Datelike, NaiveDate};
    use rspotify::model::FullTrack;

    impl TryFrom<FullTrack> for Track {
        type Error = anyhow::Error;

        fn try_from(value: FullTrack) -> Result<Self> {
            let artist_names = value
                .artists
                .iter()
                .map(|artist| artist.name.clone())
                .collect::<Vec<String>>()
                .join(", ");
            let year = match value.album.release_date {
                None => bail!("Missing release date for track: {}", value.name),
                Some(ref date_string) if date_string.is_empty() => {
                    bail!("Empty release date for track: {}", value.name)
                }
                Some(ref date_string) => {
                    // Spotify can return dates in "YYYY-MM-DD" or "YYYY" format
                    // Sometimes the year can be "0000" which is invalid
                    if date_string.contains('-') {
                        let date = date_string.parse::<NaiveDate>().context(format!("Invalid date format {date_string}"))?;
                        date.year()
                    } else {
                        let year = date_string.parse::<i32>().context(format!("Invalid year format {date_string}"))?;
                        if year == 0 {
                            bail!("Year cannot be zero for track: {}", value.name);
                        }
                        year
                    }
                }
            };
            let spotify_url = match value.external_urls.get("spotify") {
                None => bail!("Missing Spotify URL for track: {}", value.name),
                Some(url) => url.clone(),
            };

            Ok(Track {
                title: value.name,
                artist: artist_names,
                year,
                spotify_url,
                album_cover_url: value.album.images.first().map(|img| img.url.clone()),
            })
        }
    }
}
