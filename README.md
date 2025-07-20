# HighPass

<div align="center">
  <img width="1230" height="1247" alt="HighPass TUI Music Player Screenshot" src="https://github.com/user-attachments/assets/96db3f4b-0e01-40f6-b600-8016456c8c3d" />
</div>

A terminal-based music player written in **Rust** that connects to
[Subsonic](https://www.subsonic.org/)-compatible music servers. HighPass
provides a clean TUI (Terminal User Interface) for browsing your music library
and streaming audio directly in your terminal.

## Features

- **🎵 TUI Interface**: Clean, responsive terminal interface built with [ratatui](https://ratatui.rs/)
- **🎧 MPV Integration**: High-quality audio playback using libmpv
- **🌐 Subsonic Compatible**: Works with Subsonic, Navidrome, Airsonic, and other compatible servers
- **📁 Library Browser**: Collapsible tree view for Artists → Albums → Songs
- **🎨 ASCII Art**: Album cover art represented in ASCII
- **📝 Lyrics Display**: Shows song lyrics when available
- **⚙️ Configurable**: TOML-based configuration with flexible file locations

## Requirements

- **Nix** (recommended) or manual installation of:
  - Rust toolchain
  - MPV library and development headers
  - pkg-config

## Installation & Usage

### Using Nix (Recommended)

```bash
# Run directly
nix run github:pinpox/highpass

# Run locally checked out repository
nix run

# Run in development environment
nix develop --command cargo run

# Or build and run
nix develop --command bash -c 'cargo build && cargo run'
```

## Configuration

HighPass requires a configuration file to connect to your music server. The
application searches for `highpass.toml` in the following locations (in order):

1. `./highpass.toml` (current directory)
2. `~/.config/highpass/highpass.toml` (user config directory)

### Configuration Format

Create a `highpass.toml` file with your server details:

```toml
[subsonic]
server = "https://your-music-server.com"
username = "your-username"
password = "your-password"
```

An example configuration with demo server credentials is included in the repository.

### Subsonic-Compatible Servers

HighPass works with various Subsonic-compatible music servers:

- [Subsonic](https://www.subsonic.org/) - The original music streaming server
- [Navidrome](https://www.navidrome.org/) - Modern, fast, self-hosted music server
- [Airsonic](https://airsonic.github.io/) - Free, web-based media streamer
- [Airsonic-Advanced](https://github.com/airsonic-advanced/airsonic-advanced) - Community-driven fork
- [Gonic](https://github.com/sentriz/gonic) - Lightweight Subsonic server

## Command Line Options

HighPass supports several command-line flags for different use cases:

### `--info`
Display version and system information, test MPV integration:

```bash
cargo run -- --info
```

This shows:
- Application and library versions
- System MPV version and compatibility
- Runtime MPV initialization status
- Environment configuration details

### `--debug`
Enable debug logging to `./highpass.log`:

```bash
cargo run -- --debug
```

Debug mode provides detailed logging including:
- Configuration loading process
- MPV initialization and audio setup
- Subsonic API communication
- HTTP requests and responses
- Application state changes

### `--force-run`
Bypass TTY detection (useful for testing in non-terminal environments):

```bash
cargo run -- --force-run
```

## Controls

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate library tree |
| `←`/`→` | Collapse/expand tree items |
| `Enter` | Select song or expand item |
| `Space` | Play/pause current track |
| `q`/`Esc` | Quit application |

## Architecture

HighPass is built with:

- **[ratatui](https://ratatui.rs/)** - Terminal user interface framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **[libmpv-sys](https://github.com/ParadoxSpiral/libmpv-rs)** - Direct MPV library bindings for audio playback
- **[tokio](https://tokio.rs/)** - Async runtime for network operations
- **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client for Subsonic API
- **[serde](https://serde.rs/)** - Serialization framework for configuration and API responses

### Debugging

When troubleshooting issues, use the `--debug` flag to generate detailed logs:

```bash
cargo run -- --debug
# Check ./highpass.log for detailed information
```

## License

This project is licensed under the GPL-3.0+ License - see the LICENSE file for details.
