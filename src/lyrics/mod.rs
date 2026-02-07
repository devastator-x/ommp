use std::fs;
use std::path::PathBuf;

use crossbeam_channel::Sender;

use crate::event::Event;

#[derive(Debug, Clone)]
pub enum LyricsResult {
    Found { track_idx: usize, lyrics: String },
    NotFound { track_idx: usize },
    Error { track_idx: usize, msg: String },
}

pub fn spawn_fetch(
    sender: Sender<Event>,
    artist: String,
    title: String,
    album: String,
    duration_secs: f64,
    track_idx: usize,
) {
    std::thread::spawn(move || {
        let result = fetch_lyrics(&artist, &title, &album, duration_secs, track_idx);
        let _ = sender.send(Event::Lyrics(result));
    });
}

fn fetch_lyrics(
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
    track_idx: usize,
) -> LyricsResult {
    // Check cache first
    if let Some(cached) = read_cache(artist, title) {
        return LyricsResult::Found {
            track_idx,
            lyrics: cached,
        };
    }

    // Exact match requires both artist and title
    if !artist.is_empty() && !title.is_empty() {
        if let Some(lyrics) = try_get_api(artist, title, album, duration_secs) {
            write_cache(artist, title, &lyrics);
            return LyricsResult::Found { track_idx, lyrics };
        }
    }

    // Fuzzy search works with partial info (title only, or both)
    if let Some(lyrics) = try_search_api(artist, title) {
        write_cache(artist, title, &lyrics);
        return LyricsResult::Found { track_idx, lyrics };
    }

    LyricsResult::NotFound { track_idx }
}

/// LRCLIB exact match: GET /api/get?artist_name=X&track_name=Y&album_name=Z&duration=N
fn try_get_api(artist: &str, title: &str, album: &str, duration_secs: f64) -> Option<String> {
    let mut url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}",
        urlencode(artist),
        urlencode(title),
    );
    if !album.is_empty() {
        url.push_str(&format!("&album_name={}", urlencode(album)));
    }
    if duration_secs > 0.0 {
        url.push_str(&format!("&duration={}", duration_secs as u64));
    }

    let resp = ureq::get(&url)
        .set("User-Agent", "ommp/0.1.0 (https://github.com/ommp)")
        .call()
        .ok()?;

    let json: serde_json::Value = resp.into_json().ok()?;
    extract_lyrics(&json)
}

/// LRCLIB fuzzy search: GET /api/search?q=artist+title
fn try_search_api(artist: &str, title: &str) -> Option<String> {
    let query = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{} {}", artist, title)
    };
    let url = format!(
        "https://lrclib.net/api/search?q={}",
        urlencode(&query),
    );

    let resp = ureq::get(&url)
        .set("User-Agent", "ommp/0.1.0 (https://github.com/ommp)")
        .call()
        .ok()?;

    let json: serde_json::Value = resp.into_json().ok()?;
    let arr = json.as_array()?;

    // Take the first result that has lyrics
    for item in arr {
        if let Some(lyrics) = extract_lyrics(item) {
            return Some(lyrics);
        }
    }
    None
}

fn extract_lyrics(json: &serde_json::Value) -> Option<String> {
    let text = json["plainLyrics"]
        .as_str()
        .or_else(|| json["syncedLyrics"].as_str())?;
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}

fn cache_dir() -> PathBuf {
    let mut dir = dirs_config_path();
    dir.push("ommp");
    dir.push("lyrics");
    dir
}

fn cache_filename(artist: &str, title: &str) -> String {
    let sanitized = format!("{}_{}", artist, title)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect::<String>();
    format!("{}.txt", sanitized)
}

fn read_cache(artist: &str, title: &str) -> Option<String> {
    let path = cache_dir().join(cache_filename(artist, title));
    fs::read_to_string(path).ok()
}

fn write_cache(artist: &str, title: &str, lyrics: &str) {
    let dir = cache_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(cache_filename(artist, title));
    let _ = fs::write(path, lyrics);
}

fn dirs_config_path() -> PathBuf {
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from(".")
}

fn urlencode(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", b));
            }
        }
    }
    result
}
