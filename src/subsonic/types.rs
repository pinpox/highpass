use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(rename = "albumCount")]
    pub album_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
    pub year: Option<u32>,
    #[serde(rename = "songCount")]
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    #[serde(rename = "coverArt")]
    pub cover_art: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    #[serde(rename = "albumId")]
    pub album_id: Option<String>,
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
    pub track: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    #[serde(rename = "coverArt")]
    pub cover_art: Option<String>,
    pub size: Option<u64>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    pub duration: Option<u32>,
    #[serde(rename = "bitRate")]
    pub bit_rate: Option<u32>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: T,
}

#[derive(Debug, Deserialize)]
pub struct ArtistsResponse {
    pub artists: ArtistsIndex,
}

#[derive(Debug, Deserialize)]
pub struct ArtistsIndex {
    pub index: Vec<ArtistIndex>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistIndex {
    #[allow(dead_code)]
    pub name: String,
    pub artist: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumResponse {
    pub album: AlbumDetail,
}

#[derive(Debug, Deserialize)]
pub struct AlbumDetail {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub artist: Option<String>,
    #[serde(rename = "artistId")]
    #[allow(dead_code)]
    pub artist_id: Option<String>,
    #[serde(rename = "coverArt")]
    #[allow(dead_code)]
    pub cover_art: Option<String>,
    #[serde(rename = "songCount")]
    #[allow(dead_code)]
    pub song_count: Option<u32>,
    #[allow(dead_code)]
    pub duration: Option<u32>,
    #[allow(dead_code)]
    pub year: Option<u32>,
    pub song: Vec<Song>,
}

#[derive(Debug, Deserialize)]
pub struct LyricsResponse {
    pub lyrics: Option<Lyrics>,
}

#[derive(Debug, Deserialize)]
pub struct Lyrics {
    #[allow(dead_code)]
    pub artist: Option<String>,
    #[allow(dead_code)]
    pub title: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}