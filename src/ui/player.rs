use crate::subsonic::Song;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::ffi::{CString, CStr};
use std::ptr;

// Simple MPV wrapper using libmpv-sys directly
pub struct SimpleMpv {
    handle: *mut libmpv_sys::mpv_handle,
}

impl SimpleMpv {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let handle = libmpv_sys::mpv_create();
            if handle.is_null() {
                return Err("Failed to create MPV handle".into());
            }
            
            // Skip version check by initializing directly
            let ret = libmpv_sys::mpv_initialize(handle);
            if ret < 0 {
                libmpv_sys::mpv_destroy(handle);
                return Err(format!("Failed to initialize MPV: {}", ret).into());
            }
            
            Ok(SimpleMpv { handle })
        }
    }
    
    pub fn set_property(&self, name: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let name_c = CString::new(name)?;
            let value_c = CString::new(value)?;
            let ret = libmpv_sys::mpv_set_property_string(self.handle, name_c.as_ptr(), value_c.as_ptr());
            if ret < 0 {
                return Err(format!("Failed to set property {}: {}", name, ret).into());
            }
            Ok(())
        }
    }
    
    pub fn get_property<T>(&self, name: &str) -> Result<T, Box<dyn std::error::Error>> 
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        unsafe {
            let name_c = CString::new(name)?;
            let result = libmpv_sys::mpv_get_property_string(self.handle, name_c.as_ptr());
            if result.is_null() {
                return Err(format!("Failed to get property: {}", name).into());
            }
            
            let c_str = CStr::from_ptr(result);
            let str_value = c_str.to_str()?;
            let parsed_value = str_value.parse::<T>()?;
            
            libmpv_sys::mpv_free(result as *mut _);
            Ok(parsed_value)
        }
    }
    
    pub fn command(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let mut c_args: Vec<CString> = Vec::new();
            c_args.push(CString::new(command)?);
            for arg in args {
                c_args.push(CString::new(*arg)?);
            }
            
            let mut c_arg_ptrs: Vec<*const i8> = c_args.iter().map(|s| s.as_ptr()).collect();
            c_arg_ptrs.push(ptr::null());
            
            let ret = libmpv_sys::mpv_command(self.handle, c_arg_ptrs.as_mut_ptr());
            if ret < 0 {
                return Err(format!("Command failed: {}", ret).into());
            }
            Ok(())
        }
    }
}

impl Drop for SimpleMpv {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                libmpv_sys::mpv_destroy(self.handle);
            }
        }
    }
}

// Make it safe to send between threads
unsafe impl Send for SimpleMpv {}

type Mpv = SimpleMpv;
use log::{info, warn, error, debug};

// SimpleMpv is already defined above and exported via the module

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    Previous,
    PlayPause,
    Next,
    Seek(f32),
}

pub struct PlayerState {
    pub current_song: Option<Song>,
    pub is_playing: bool,
    pub progress: f32,
    pub duration: f32,
    pub cover_art: Option<Vec<u8>>,
    pub lyrics: Option<String>,
    pub mpv: Option<Mpv>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_song: None,
            is_playing: false,
            progress: 0.0,
            duration: 100.0,
            cover_art: None,
            lyrics: None,
            mpv: None,
        }
    }
}

impl PlayerState {
    pub fn new() -> Self {
        info!("Initializing PlayerState");
        let mut state = Self::default();
        
        // Initialize MPV
        info!("Attempting to initialize MPV");
        match Mpv::new() {
            Ok(mpv) => {
                info!("Successfully initialized MPV");
                
                // Set some basic properties
                if let Err(e) = mpv.set_property("vid", "no") {
                    warn!("Failed to disable video: {}", e);
                }
                
                if let Err(e) = mpv.set_property("audio-client-name", "HighPass") {
                    warn!("Failed to set audio client name: {}", e);
                }
                
                // Log MPV version information
                match mpv.get_property::<String>("mpv-version") {
                    Ok(version) => info!("MPV version: {}", version),
                    Err(e) => debug!("Could not get MPV version: {}", e),
                }
                
                state.mpv = Some(mpv);
                info!("MPV integration enabled");
            }
            Err(e) => {
                error!("Failed to initialize MPV: {}", e);
                warn!("Running in UI-only mode - audio playback will not be available");
                warn!("The application will still work for browsing music and displaying metadata");
            }
        }
        
        state
    }

    pub fn set_current_song(&mut self, song: Song) {
        info!("Setting current song: {} by {}", 
               song.title, 
               song.artist.as_deref().unwrap_or("Unknown Artist"));
        
        self.current_song = Some(song);
        self.progress = 0.0;
        
        if let Some(duration) = &self.current_song.as_ref().unwrap().duration {
            self.duration = *duration as f32;
            debug!("Song duration: {} seconds", self.duration);
        }
    }

    pub fn play_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting to play URL: {}", url);
        
        if let Some(mpv) = &self.mpv {
            debug!("Sending loadfile command to MPV");
            match mpv.command("loadfile", &[url]) {
                Ok(_) => {
                    info!("Successfully sent loadfile command to MPV");
                    self.is_playing = true;
                    
                    // Try to get some info about the loaded file
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    match mpv.get_property::<String>("media-title") {
                        Ok(title) => info!("MPV media title: {}", title),
                        Err(e) => debug!("Could not get media title: {}", e),
                    }
                    
                    match mpv.get_property::<f64>("duration") {
                        Ok(duration) => info!("MPV duration: {} seconds", duration),
                        Err(e) => debug!("Could not get duration: {}", e),
                    }
                }
                Err(e) => {
                    error!("Failed to send loadfile command to MPV: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            warn!("MPV not available - simulating playback for UI testing");
            warn!("Would play: {}", url);
            self.is_playing = true; // Simulate playback state for UI
        }
        
        Ok(())
    }

    pub fn set_cover_art(&mut self, cover_art: Vec<u8>) {
        self.cover_art = Some(cover_art);
    }

    pub fn set_lyrics(&mut self, lyrics: String) {
        self.lyrics = Some(lyrics);
    }

    pub fn toggle_play_pause(&mut self) {
        if let Some(mpv) = &self.mpv {
            if self.is_playing {
                info!("Pausing playback");
                if let Err(e) = mpv.set_property("pause", "yes") {
                    error!("Failed to pause: {}", e);
                } else {
                    self.is_playing = false;
                    debug!("Successfully paused");
                }
            } else {
                info!("Resuming playback");
                if let Err(e) = mpv.set_property("pause", "no") {
                    error!("Failed to resume: {}", e);
                } else {
                    self.is_playing = true;
                    debug!("Successfully resumed");
                }
            }
        } else {
            warn!("Cannot toggle play/pause - MPV not initialized");
        }
    }

    pub fn stop(&mut self) {
        if let Some(mpv) = &self.mpv {
            let _ = mpv.command("stop", &[""]);
            self.is_playing = false;
            self.progress = 0.0;
        }
    }

    pub fn seek(&mut self, position: f32) {
        if let Some(mpv) = &self.mpv {
            let _ = mpv.command("seek", &[&position.to_string(), "absolute"]);
            self.progress = position;
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }

    pub fn update_progress(&mut self) {
        if let Some(mpv) = &self.mpv {
            match mpv.get_property::<f64>("time-pos") {
                Ok(time_pos) => {
                    let new_progress = time_pos as f32;
                    if (new_progress - self.progress).abs() > 1.0 { // Only log every second
                        debug!("Progress: {:.1}s", new_progress);
                    }
                    self.progress = new_progress;
                }
                Err(e) => {
                    if e.to_string() != "property unavailable" {
                        debug!("Could not get time-pos: {}", e);
                    }
                }
            }
            
            if let Ok(duration) = mpv.get_property::<f64>("duration") {
                let new_duration = duration as f32;
                if (new_duration - self.duration).abs() > 0.1 {
                    debug!("Duration updated: {:.1}s", new_duration);
                    self.duration = new_duration;
                }
            }
            
            match mpv.get_property::<bool>("pause") {
                Ok(pause) => {
                    let was_playing = self.is_playing;
                    self.is_playing = !pause;
                    if was_playing != self.is_playing {
                        info!("Playback state changed: {}", if self.is_playing { "playing" } else { "paused" });
                    }
                }
                Err(e) => {
                    debug!("Could not get pause state: {}", e);
                }
            }
        }
    }
}

pub struct PlayerWidget;

impl PlayerWidget {
    pub fn render(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Track info
                Constraint::Min(10),   // Cover art
                Constraint::Min(8),    // Lyrics
                Constraint::Length(3), // Progress bar
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Track info
        Self::render_track_info(state, layout[0], buf);
        
        // Cover art
        Self::render_cover_art(state, layout[1], buf);
        
        // Lyrics
        Self::render_lyrics(state, layout[2], buf);
        
        // Progress bar
        Self::render_progress_bar(state, layout[3], buf);
        
        // Controls
        Self::render_controls(state, layout[4], buf);
    }

    fn render_track_info(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let text = if let Some(song) = &state.current_song {
            format!(
                "♪ {} - {} ({})",
                song.title,
                song.artist.as_deref().unwrap_or("Unknown Artist"),
                song.album.as_deref().unwrap_or("Unknown Album")
            )
        } else {
            "No track selected".to_string()
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Now Playing").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }

    fn render_cover_art(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let ascii_art = if state.cover_art.is_some() {
            vec![
                "┌─────────────────────┐",
                "│ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪ │",
                "│ ♫     ALBUM      ♫ │",
                "│ ♪     COVER      ♪ │",
                "│ ♫      ART       ♫ │",
                "│ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪ │",
                "└─────────────────────┘",
            ]
        } else {
            vec![
                "┌─────────────────────┐",
                "│                     │",
                "│    NO COVER ART     │",
                "│     AVAILABLE       │",
                "│                     │",
                "│         ♪           │",
                "└─────────────────────┘",
            ]
        };

        let text = ascii_art.join("\n");
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Cover Art").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }

    fn render_lyrics(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let lyrics_text = if let Some(lyrics) = &state.lyrics {
            lyrics.lines().take(10).collect::<Vec<_>>().join("\n")
        } else {
            "No lyrics available".to_string()
        };

        let paragraph = Paragraph::new(lyrics_text)
            .block(Block::default().title("Lyrics").borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow))
            .wrap(ratatui::widgets::Wrap { trim: true });

        paragraph.render(area, buf);
    }

    fn render_progress_bar(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let progress_ratio = if state.duration > 0.0 {
            (state.progress / state.duration).min(1.0)
        } else {
            0.0
        };

        let current_time = Self::format_time(state.progress);
        let total_time = Self::format_time(state.duration);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .percent((progress_ratio * 100.0) as u16)
            .label(format!("{} / {}", current_time, total_time));

        gauge.render(area, buf);
    }

    fn render_controls(state: &PlayerState, area: Rect, buf: &mut Buffer) {
        let play_pause_symbol = if state.is_playing { "⏸" } else { "▶" };
        let controls_text = format!("⏮  {}  ⏭  [Space: Play/Pause, ←/→: Prev/Next]", play_pause_symbol);

        let paragraph = Paragraph::new(controls_text)
            .block(Block::default().title("Controls").borders(Borders::ALL))
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }

    fn format_time(seconds: f32) -> String {
        let total_seconds = seconds as u32;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}