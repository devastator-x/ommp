# OMMP - Oh My Music Player

A terminal-based music player built with Rust. Plays FLAC, MP3, M4A, OGG, and WAV files directly from your `~/Music` directory.

Inspired by [rmpc](https://github.com/mierak/rmpc), but fully standalone — no MPD required.

## Features

- **Direct playback** of FLAC, MP3, M4A/AAC, OGG, WAV
- **3-column layout** with resizable panes (Library | Queue | Info)
- **Album art** display with native terminal protocol detection (Kitty / Sixel / HalfBlocks)
- **6 browsing tabs**: Queue, Directories, Artists, Albums, Genre, Playlists
- **Playlist management** with bookmarks and persistent state
- **Global search** modal with fuzzy matching
- **Mouse support** with hover highlighting, click-to-select, and drag-to-resize
- **Shuffle, repeat** (off / all / one), volume control
- **Track metadata** display (title, artist, album, genre, bitrate, format, etc.)
- **ASCII clock** widget with gradient colors
- **Vibrant color scheme** with per-element RGB styling
- **Nerd Font icons** throughout the UI
- **State persistence** across sessions (volume, shuffle, repeat, playlists, layout)

## Requirements

- Rust 1.70+
- Linux (PulseAudio or ALSA)
- A terminal with true color support
- [Nerd Font](https://www.nerdfonts.com/) for icons (e.g., MesloLGS NF, JetBrainsMono Nerd Font)
- For high-quality album art: [Kitty](https://sw.kovidgoyal.net/kitty/) or a Sixel-capable terminal

## Install

```bash
git clone https://github.com/devastator-x/ommp.git
cd ommp
cargo build --release
./target/release/ommp
```

## Keybindings

### Playback

| Key | Action |
|-----|--------|
| `Space` | Play / Pause |
| `n` | Next track |
| `N` | Previous track |
| `+` / `-` | Volume up / down |
| `Left` / `Right` | Seek backward / forward (5s) |
| `s` | Toggle shuffle |
| `r` | Cycle repeat (off / all / one) |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Move down / up |
| `h` / `l` | Focus previous / next pane |
| `Tab` / `Shift+Tab` | Cycle pane focus |
| `1`-`6` | Switch to tab |
| `Enter` | Play selected / Add to queue |
| `g` / `G` | Jump to top / bottom |

### UI

| Key | Action |
|-----|--------|
| `Ctrl+E, r` | Toggle resize mode (then `h/l/j/k` to resize) |
| `Ctrl+E, s` | Open search modal |
| `Ctrl+E, h` | Open help modal |
| `Ctrl+E, i` | Open about modal |
| `p` | Cycle info pane view (Clock / Album Art) |
| `b` | Open playlist/bookmark modal |
| `q` | Quit |

## Tech Stack

| Role | Crate |
|------|-------|
| TUI rendering | `ratatui` 0.29 |
| Terminal control | `crossterm` 0.28 |
| Audio playback | `rodio` 0.21 + Symphonia |
| Metadata parsing | `lofty` 0.21 |
| Album art | `ratatui-image` 4.2 |
| File scanning | `walkdir` 2 |

## License

MIT
