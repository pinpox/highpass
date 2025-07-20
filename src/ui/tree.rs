use crate::subsonic::{Artist, Album, Song};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TreeMessage {
    ToggleArtist(String),
    ToggleAlbum(String),
    SelectSong(Song),
    LoadArtistAlbums(String),
    LoadAlbumSongs(String),
}

#[derive(Debug, Clone)]
pub struct TreeState {
    pub artists: Vec<Artist>,
    pub expanded_artists: HashMap<String, bool>,
    pub expanded_albums: HashMap<String, bool>,
    pub artist_albums: HashMap<String, Vec<Album>>,
    pub album_songs: HashMap<String, Vec<Song>>,
    pub selected_song: Option<Song>,
    pub list_state: ListState,
    pub items: Vec<TreeItem>,
}

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub id: String,
    pub display_text: String,
    pub item_type: TreeItemType,
    pub level: usize,
}

#[derive(Debug, Clone)]
pub enum TreeItemType {
    Artist(Artist),
    Album(Album),
    Song(Song),
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            artists: Vec::new(),
            expanded_artists: HashMap::new(),
            expanded_albums: HashMap::new(),
            artist_albums: HashMap::new(),
            album_songs: HashMap::new(),
            selected_song: None,
            list_state: ListState::default(),
            items: Vec::new(),
        }
    }
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_artists(&mut self, artists: Vec<Artist>) {
        self.artists = artists;
        self.rebuild_items();
    }

    pub fn toggle_artist(&mut self, artist_id: &str) -> bool {
        let expanded = !self.expanded_artists.get(artist_id).unwrap_or(&false);
        self.expanded_artists.insert(artist_id.to_string(), expanded);
        self.rebuild_items();
        expanded && !self.artist_albums.contains_key(artist_id)
    }

    pub fn toggle_album(&mut self, album_id: &str) -> bool {
        let expanded = !self.expanded_albums.get(album_id).unwrap_or(&false);
        self.expanded_albums.insert(album_id.to_string(), expanded);
        self.rebuild_items();
        expanded && !self.album_songs.contains_key(album_id)
    }

    pub fn set_artist_albums(&mut self, artist_id: String, albums: Vec<Album>) {
        self.artist_albums.insert(artist_id, albums);
        self.rebuild_items();
    }

    pub fn set_album_songs(&mut self, album_id: String, songs: Vec<Song>) {
        self.album_songs.insert(album_id, songs);
        self.rebuild_items();
    }

    pub fn select_song(&mut self, song: Song) {
        self.selected_song = Some(song);
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => (i + 1) % self.items.len(),
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    pub fn get_selected_item(&self) -> Option<&TreeItem> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }

    fn rebuild_items(&mut self) {
        self.items.clear();
        
        for artist in &self.artists {
            let is_expanded = self.expanded_artists.get(&artist.id).unwrap_or(&false);
            let prefix = if *is_expanded { "▼" } else { "▶" };
            
            self.items.push(TreeItem {
                id: artist.id.clone(),
                display_text: format!("{} {}", prefix, artist.name),
                item_type: TreeItemType::Artist(artist.clone()),
                level: 0,
            });

            if *is_expanded {
                if let Some(albums) = self.artist_albums.get(&artist.id) {
                    for album in albums {
                        let is_album_expanded = self.expanded_albums.get(&album.id).unwrap_or(&false);
                        let prefix = if *is_album_expanded { "▼" } else { "▶" };
                        
                        self.items.push(TreeItem {
                            id: album.id.clone(),
                            display_text: format!("  {} {}", prefix, album.name),
                            item_type: TreeItemType::Album(album.clone()),
                            level: 1,
                        });

                        if *is_album_expanded {
                            if let Some(songs) = self.album_songs.get(&album.id) {
                                for song in songs {
                                    self.items.push(TreeItem {
                                        id: song.id.clone(),
                                        display_text: format!("    ♪ {}", song.title),
                                        item_type: TreeItemType::Song(song.clone()),
                                        level: 2,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct TreeWidget;

impl TreeWidget {
    pub fn render(state: &mut TreeState, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = state
            .items
            .iter()
            .map(|item| ListItem::new(item.display_text.clone()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Library").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol(">");

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}