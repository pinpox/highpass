use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use log::{info, debug};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubsonicConfig {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub subsonic: SubsonicConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            subsonic: SubsonicConfig {
                server: "http://demo.subsonic.org".to_string(),
                username: "guest".to_string(),
                password: "guest".to_string(),
            },
        }
    }
}

impl Config {
    /// Load configuration from TOML files in the specified search order:
    /// 1. ./highpass.toml
    /// 2. ~/.config/highpass/highpass.toml
    /// 
    /// Returns an error if no config file is found.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_paths = Self::get_config_paths();
        
        for path in &config_paths {
            debug!("Checking for config file at: {}", path.display());
            if path.exists() {
                info!("Loading configuration from: {}", path.display());
                return Self::load_from_file(path);
            }
        }
        
        // No config file found - fail hard
        let error_msg = format!(
            "No configuration file found. Please create a configuration file in one of the following locations:\n{}",
            config_paths.iter()
                .map(|p| format!("  {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        Err(error_msg.into())
    }
    
    /// Get the list of possible configuration file paths in search order
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // 1. ./highpass.toml (current directory)
        paths.push(PathBuf::from("./highpass.toml"));
        
        // 2. ~/.config/highpass/highpass.toml (user config directory)
        if let Some(config_dir) = Self::get_user_config_dir() {
            paths.push(config_dir.join("highpass").join("highpass.toml"));
        }
        
        paths
    }
    
    /// Get the user's config directory (~/.config on Unix)
    fn get_user_config_dir() -> Option<PathBuf> {
        #[cfg(unix)]
        {
            if let Ok(home) = std::env::var("HOME") {
                Some(PathBuf::from(home).join(".config"))
            } else {
                None
            }
        }
        
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                Some(PathBuf::from(appdata))
            } else {
                None
            }
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            None
        }
    }
    
    /// Load configuration from a specific file
    fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        
        info!("Successfully loaded configuration:");
        info!("  Server: {}", config.subsonic.server);
        info!("  Username: {}", config.subsonic.username);
        // Don't log the password for security
        info!("  Password: [configured]");
        
        Ok(config)
    }
    
}