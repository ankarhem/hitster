use anyhow::Result;
use futures_util::StreamExt;
use rspotify::{ClientCredsSpotify, Credentials, prelude::BaseClient};
use rspotify::model::PlayableItem;
use crate::Settings;
use tracing::{info, instrument};
use crate::application::ISpotifyClient;
use crate::domain::{Playlist, PlaylistId, SpotifyId};

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
    async fn get_playlist(&self, id: &SpotifyId) -> Result<Option<Playlist>> {
        let spotify_id = id.to_string();
        let rspotify_playlist_id = rspotify::model::PlaylistId::from_id_or_uri(&spotify_id)?;
        let full_playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;
        let mut tracks_stream = self.client.playlist_items(full_playlist.id, None, None);

        let mut tracks = Vec::new();
        let mut skipped_tracks = 0;

        while let Some(item_result) = tracks_stream.next().await {
            let item = item_result?;

            if let Some(PlayableItem::Track(track)) = item.track {
                if let Ok(track) = track.try_into() {
                    tracks.push(track);
                    continue;
                }
            }
            skipped_tracks += 1;
        }

        if skipped_tracks > 0 {
            tracing::warn!("Skipped {} tracks", skipped_tracks);
        }

        Ok(Some(Playlist {
            id: PlaylistId::new()?,
            name: full_playlist.name,
            tracks,
            spotify_id: Some(id.clone()),
        }))
    }
}

mod conversions {
    use rspotify::model::FullTrack;
    use crate::domain::Track;
    use anyhow::{bail, Result};
    use chrono::{Datelike, NaiveDate};

    impl TryFrom<FullTrack> for Track {
        type Error = anyhow::Error;

        fn try_from(value: FullTrack) -> Result<Self> {
            let artist_names = value.artists.iter().map(|a| a.name.clone()).collect();
            let year = match value.album.release_date {
                None => bail!("Missing release date for track: {}", value.name),
                Some(ref date_string) if date_string.is_empty() => bail!("Empty release date for track: {}", value.name),
                Some(ref date_string) => {
                    let date = date_string.parse::<NaiveDate>()?;
                    date.year()
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
                spotify_url
            })
        }
    }
}