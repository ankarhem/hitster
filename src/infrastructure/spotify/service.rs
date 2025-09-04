use anyhow::Result;
use futures_util::StreamExt;
use rspotify::{ClientCredsSpotify, Credentials, prelude::BaseClient};
use rspotify::model::PlayableItem;
use crate::application::models::{Playlist, PlaylistId};
use crate::Settings;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct SpotifyService {
    client: ClientCredsSpotify,
}

impl SpotifyService {
    #[instrument(skip(settings))]
    pub async fn new(settings: &Settings) -> Result<Self> {
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);
        let client = ClientCredsSpotify::new(creds);
        client.request_token().await?;
        info!("Spotify authentication successful");

        Ok(Self { client })
    }

    #[instrument(skip(self), fields(playlist_id = %playlist_id))]
    pub async fn get_playlist(&self, playlist_id: PlaylistId) -> Result<Playlist> {
        let rspotify_playlist_id = playlist_id.clone().try_into()?;
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

        Ok(Playlist {
            id: playlist_id.try_into()?,
            name: full_playlist.name,
            tracks,
        })
    }
}