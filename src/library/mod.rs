pub mod scanner;
pub mod track;
pub mod watcher;

use std::collections::BTreeSet;
use std::path::Path;
use track::Track;

#[derive(Debug)]
pub struct Library {
    pub tracks: Vec<Track>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
        }
    }

    pub fn scan(path: &Path) -> Self {
        let tracks = scanner::scan_directory(path);
        Self { tracks }
    }

    pub fn get_artists(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        let mut has_unknown = false;
        for t in &self.tracks {
            if t.artist.is_empty() {
                has_unknown = true;
            } else {
                set.insert(t.artist.clone());
            }
        }
        let mut result: Vec<String> = set.into_iter().collect();
        if has_unknown {
            result.push("Unknown Artist".to_string());
        }
        result
    }

    pub fn get_album_artists(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for t in &self.tracks {
            if !t.album_artist.is_empty() {
                set.insert(t.album_artist.clone());
            }
        }
        set.into_iter().collect()
    }

    pub fn get_genres(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for t in &self.tracks {
            if !t.genre.is_empty() {
                set.insert(t.genre.clone());
            }
        }
        set.into_iter().collect()
    }

    pub fn get_albums(&self) -> Vec<(String, String)> {
        let mut set = BTreeSet::new();
        for t in &self.tracks {
            if !t.album.is_empty() {
                let artist = if t.album_artist.is_empty() {
                    t.artist.clone()
                } else {
                    t.album_artist.clone()
                };
                set.insert((t.album.clone(), artist));
            }
        }
        set.into_iter().collect()
    }

    pub fn get_tracks_by_artist(&self, artist: &str) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                if artist == "Unknown Artist" {
                    t.artist.is_empty()
                } else {
                    t.artist == artist
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_tracks_by_album_artist(&self, album_artist: &str) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.album_artist == album_artist)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_tracks_by_album(&self, album: &str) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.album == album)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_tracks_by_genre(&self, genre: &str) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.genre == genre)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_albums_by_album_artist(&self, album_artist: &str) -> Vec<String> {
        let mut set = BTreeSet::new();
        for t in &self.tracks {
            if t.album_artist == album_artist && !t.album.is_empty() {
                set.insert(t.album.clone());
            }
        }
        set.into_iter().collect()
    }

    pub fn get_directory_entries(&self, dir: &Path) -> (Vec<String>, Vec<usize>) {
        let mut subdirs = BTreeSet::new();
        let mut tracks = Vec::new();

        for (i, t) in self.tracks.iter().enumerate() {
            if let Some(parent) = t.path.parent() {
                if parent == dir {
                    tracks.push(i);
                } else if let Ok(rel) = parent.strip_prefix(dir) {
                    if let Some(first) = rel.components().next() {
                        subdirs.insert(first.as_os_str().to_string_lossy().to_string());
                    }
                }
            }
        }

        (subdirs.into_iter().collect(), tracks)
    }

    pub fn path_to_index(&self, path: &Path) -> Option<usize> {
        self.tracks.iter().position(|t| t.path == path)
    }

    pub fn search(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.title.to_lowercase().contains(&q)
                    || t.artist.to_lowercase().contains(&q)
                    || t.album.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }
}
