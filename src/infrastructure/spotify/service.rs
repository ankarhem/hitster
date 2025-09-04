use anyhow::Result;
use rspotify::{ClientCredsSpotify, Credentials, prelude::BaseClient};
use crate::application::models::{Playlist, PlaylistId};
use crate::infrastructure::spotify::SpotifyMapper;
use crate::Settings;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct SpotifyService {
    client: ClientCredsSpotify,
    mapper: SpotifyMapper,
}

impl SpotifyService {
    #[instrument(skip(settings))]
    pub async fn new(settings: &Settings) -> Result<Self> {
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);
        let client = ClientCredsSpotify::new(creds);
        client.request_token().await?;
        info!("Spotify authentication successful");

        let mapper = SpotifyMapper::new(client.clone());
        Ok(Self { client, mapper })
    }

    #[instrument(skip(self), fields(playlist_id = %playlist_id))]
    pub async fn get_playlist(&self, playlist_id: PlaylistId) -> Result<Playlist> {
        let rspotify_playlist_id = playlist_id.clone().try_into()?;
        let full_playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;
        self.mapper.map_full_playlist(full_playlist).await
    }
}