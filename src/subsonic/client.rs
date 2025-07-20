use crate::subsonic::types::*;
use reqwest::Client;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SubsonicClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
}

impl SubsonicClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            base_url,
            username,
            password,
            client: Client::new(),
        }
    }

    fn build_url(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        let mut url = format!("{}/rest/{}", self.base_url, endpoint);
        let _token = format!("{:x}", md5::compute(&self.password));
        let salt = uuid::Uuid::new_v4().to_string();
        let token_hash = format!("{:x}", md5::compute(format!("{}{}", &self.password, &salt)));

        let mut query_params = vec![
            ("u", self.username.as_str()),
            ("t", &token_hash),
            ("s", &salt),
            ("v", "1.16.1"),
            ("c", "highpass"),
            ("f", "json"),
        ];

        query_params.extend_from_slice(params);

        url.push('?');
        for (i, (key, value)) in query_params.iter().enumerate() {
            if i > 0 {
                url.push('&');
            }
            url.push_str(&format!("{}={}", key, urlencoding::encode(value)));
        }

        url
    }

    pub async fn get_artists(&self) -> Result<Vec<Artist>, Box<dyn std::error::Error>> {
        let url = self.build_url("getArtists", &[]);
        let response: SubsonicResponse<ArtistsResponse> = self.client.get(&url).send().await?.json().await?;
        
        let mut artists = Vec::new();
        for index in response.subsonic_response.artists.index {
            artists.extend(index.artist);
        }
        
        Ok(artists)
    }

    pub async fn get_artist(&self, artist_id: &str) -> Result<Vec<Album>, Box<dyn std::error::Error>> {
        let url = self.build_url("getArtist", &[("id", artist_id)]);
        let response: Value = self.client.get(&url).send().await?.json().await?;
        
        let albums = response
            .get("subsonic-response")
            .and_then(|r| r.get("artist"))
            .and_then(|a| a.get("album"))
            .and_then(|a| a.as_array())
            .map(|albums| {
                albums
                    .iter()
                    .filter_map(|album| serde_json::from_value(album.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(albums)
    }

    pub async fn get_album(&self, album_id: &str) -> Result<AlbumDetail, Box<dyn std::error::Error>> {
        let url = self.build_url("getAlbum", &[("id", album_id)]);
        let response: SubsonicResponse<AlbumResponse> = self.client.get(&url).send().await?.json().await?;
        Ok(response.subsonic_response.album)
    }

    pub async fn get_cover_art(&self, cover_art_id: &str, size: Option<u32>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let size_param = size.map(|s| s.to_string()).unwrap_or_else(|| "200".to_string());
        let url = self.build_url("getCoverArt", &[("id", cover_art_id), ("size", &size_param)]);
        let response = self.client.get(&url).send().await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn get_lyrics(&self, artist: &str, title: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let url = self.build_url("getLyrics", &[("artist", artist), ("title", title)]);
        let response: SubsonicResponse<LyricsResponse> = self.client.get(&url).send().await?.json().await?;
        Ok(response.subsonic_response.lyrics.and_then(|l| l.text))
    }

    pub fn get_stream_url(&self, song_id: &str) -> String {
        self.build_url("stream", &[("id", song_id)])
    }
}