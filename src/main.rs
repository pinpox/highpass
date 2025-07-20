mod subsonic;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    tty::IsTty,
};
use ratatui::prelude::*;
use std::{error::Error, io, time::Duration};
use subsonic::{SubsonicClient, Artist, Album, Song};
use ui::{
    tree::{TreeWidget, TreeState, TreeItemType},
    player::{PlayerWidget, PlayerState},
};
use tokio::sync::mpsc;
use log::{info, warn, error, debug};

#[derive(Debug, Clone)]
pub enum Message {
    LoadedArtists(Vec<Artist>),
    LoadedArtistAlbums(String, Vec<Album>),
    LoadedAlbumSongs(String, Vec<Song>),
    LoadedCoverArt(Vec<u8>),
    LoadedLyrics(String),
    Quit,
}

pub struct App {
    subsonic_client: Option<SubsonicClient>,
    tree_state: TreeState,
    player_state: PlayerState,
    should_quit: bool,
    message_receiver: mpsc::UnboundedReceiver<Message>,
    message_sender: mpsc::UnboundedSender<Message>,
}

impl App {
    pub fn new() -> Self {
        info!("Initializing HighPass application");
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        let mut app = Self {
            subsonic_client: None,
            tree_state: TreeState::new(),
            player_state: PlayerState::new(),
            should_quit: false,
            message_receiver,
            message_sender,
        };

        // Initialize with demo data for development
        info!("Connecting to Subsonic demo server");
        let demo_client = SubsonicClient::new(
            "http://demo.subsonic.org".to_string(),
            "guest".to_string(),
            "guest".to_string(),
        );
        app.subsonic_client = Some(demo_client);

        // Load artists asynchronously
        info!("Loading artists from Subsonic server");
        let client = app.subsonic_client.as_ref().unwrap().clone();
        let sender = app.message_sender.clone();
        tokio::spawn(async move {
            match client.get_artists().await {
                Ok(artists) => {
                    info!("Successfully loaded {} artists", artists.len());
                    let _ = sender.send(Message::LoadedArtists(artists));
                }
                Err(e) => {
                    error!("Failed to load artists: {}", e);
                }
            }
        });

        app
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            // Handle async messages
            while let Ok(message) = self.message_receiver.try_recv() {
                self.handle_message(message).await;
            }

            if self.should_quit {
                break;
            }

            // Update player progress
            self.player_state.update_progress();

            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key.code).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::LoadedArtists(artists) => {
                self.tree_state.set_artists(artists);
            }
            Message::LoadedArtistAlbums(artist_id, albums) => {
                self.tree_state.set_artist_albums(artist_id, albums);
            }
            Message::LoadedAlbumSongs(album_id, songs) => {
                self.tree_state.set_album_songs(album_id, songs);
            }
            Message::LoadedCoverArt(cover_art) => {
                self.player_state.set_cover_art(cover_art);
            }
            Message::LoadedLyrics(lyrics) => {
                self.player_state.set_lyrics(lyrics);
            }
            Message::Quit => {
                self.should_quit = true;
            }
        }
    }

    async fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Up => {
                self.tree_state.previous();
            }
            KeyCode::Down => {
                self.tree_state.next();
            }
            KeyCode::Enter | KeyCode::Right => {
                if let Some(item) = self.tree_state.get_selected_item().cloned() {
                    match &item.item_type {
                        TreeItemType::Artist(artist) => {
                            let should_load = self.tree_state.toggle_artist(&artist.id);
                            if should_load {
                                self.load_artist_albums(artist.id.clone()).await;
                            }
                        }
                        TreeItemType::Album(album) => {
                            let should_load = self.tree_state.toggle_album(&album.id);
                            if should_load {
                                self.load_album_songs(album.id.clone()).await;
                            }
                        }
                        TreeItemType::Song(song) => {
                            self.select_song(song.clone()).await;
                        }
                    }
                }
            }
            KeyCode::Left => {
                if let Some(item) = self.tree_state.get_selected_item().cloned() {
                    match &item.item_type {
                        TreeItemType::Artist(artist) => {
                            self.tree_state.toggle_artist(&artist.id);
                        }
                        TreeItemType::Album(album) => {
                            self.tree_state.toggle_album(&album.id);
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char(' ') => {
                self.player_state.toggle_play_pause();
            }
            _ => {}
        }
    }

    async fn load_artist_albums(&self, artist_id: String) {
        if let Some(client) = &self.subsonic_client {
            let client = client.clone();
            let sender = self.message_sender.clone();
            tokio::spawn(async move {
                if let Ok(albums) = client.get_artist(&artist_id).await {
                    let _ = sender.send(Message::LoadedArtistAlbums(artist_id, albums));
                }
            });
        }
    }

    async fn load_album_songs(&self, album_id: String) {
        if let Some(client) = &self.subsonic_client {
            let client = client.clone();
            let sender = self.message_sender.clone();
            tokio::spawn(async move {
                if let Ok(album_detail) = client.get_album(&album_id).await {
                    let _ = sender.send(Message::LoadedAlbumSongs(album_id, album_detail.song));
                }
            });
        }
    }

    async fn select_song(&mut self, song: Song) {
        info!("User selected song: {} by {}", 
               song.title, 
               song.artist.as_deref().unwrap_or("Unknown Artist"));
        
        self.player_state.set_current_song(song.clone());
        self.tree_state.select_song(song.clone());

        if let Some(client) = &self.subsonic_client {
            // Start playing the song
            let stream_url = client.get_stream_url(&song.id);
            info!("Generated stream URL: {}", stream_url);
            
            match self.player_state.play_url(&stream_url) {
                Ok(_) => {
                    info!("Successfully initiated playback");
                }
                Err(e) => {
                    error!("Failed to play song: {}", e);
                }
            }

            // Load cover art
            if let Some(cover_art_id) = &song.cover_art {
                debug!("Loading cover art with ID: {}", cover_art_id);
                let client_clone = client.clone();
                let cover_art_id = cover_art_id.clone();
                let sender = self.message_sender.clone();
                tokio::spawn(async move {
                    match client_clone.get_cover_art(&cover_art_id, Some(200)).await {
                        Ok(cover_art) => {
                            debug!("Successfully loaded cover art ({} bytes)", cover_art.len());
                            let _ = sender.send(Message::LoadedCoverArt(cover_art));
                        }
                        Err(e) => {
                            warn!("Failed to load cover art: {}", e);
                        }
                    }
                });
            } else {
                debug!("No cover art ID available for this song");
            }

            // Load lyrics
            if let (Some(artist), title) = (&song.artist, &song.title) {
                debug!("Loading lyrics for: {} - {}", artist, title);
                let client_clone = client.clone();
                let artist = artist.clone();
                let title = title.clone();
                let sender = self.message_sender.clone();
                tokio::spawn(async move {
                    match client_clone.get_lyrics(&artist, &title).await {
                        Ok(Some(lyrics)) => {
                            debug!("Successfully loaded lyrics ({} chars)", lyrics.len());
                            let _ = sender.send(Message::LoadedLyrics(lyrics));
                        }
                        Ok(None) => {
                            debug!("No lyrics available for this song");
                        }
                        Err(e) => {
                            warn!("Failed to load lyrics: {}", e);
                        }
                    }
                });
            } else {
                debug!("Missing artist or title for lyrics lookup");
            }
        } else {
            error!("No Subsonic client available");
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(f.area());

        // Tree view (left panel)
        TreeWidget::render(&mut self.tree_state, layout[0], f.buffer_mut());

        // Player view (right panel)
        PlayerWidget::render(&self.player_state, layout[1], f.buffer_mut());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stderr)
        .init();
    
    info!("Starting HighPass music player");

    // Create app first to test MPV initialization
    info!("Creating application instance");
    let mut app = App::new();
    
    // Give some time for async initialization
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    info!("Application created successfully");

    // Check if we're in a proper terminal environment
    if !IsTty::is_tty(&io::stdout()) {
        error!("Not running in a TTY environment.");
        error!("This TUI application requires a proper terminal.");
        error!("Please run this application in a regular terminal session.");
        return Err("Not a TTY".into());
    }

    // Setup terminal
    info!("Setting up terminal");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    info!("Enabled raw mode");
    
    execute!(stdout, EnterAlternateScreen)?;
    info!("Entered alternate screen");
    
    // Skip mouse capture for now as it might be causing issues
    // execute!(stdout, EnableMouseCapture)?;
    info!("Skipping mouse capture for compatibility");
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    info!("Terminal setup complete");

    // Run the app
    info!("Starting main application loop");
    let res = app.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
        // DisableMouseCapture  // Skip since we didn't enable it
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}