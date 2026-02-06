#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            RepeatMode::Off => "\u{21BB}",  // ↻
            RepeatMode::All => "\u{21BB}",  // ↻
            RepeatMode::One => "\u{21BB}1", // ↻1
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Queue,
    Directories,
    Artists,
    AlbumArtists,
    Albums,
    Genre,
    Playlists,
    Search,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::Queue,
        Tab::Directories,
        Tab::Artists,
        Tab::AlbumArtists,
        Tab::Albums,
        Tab::Genre,
        Tab::Playlists,
        Tab::Search,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Queue => "Queue",
            Tab::Directories => "Directories",
            Tab::Artists => "Artists",
            Tab::AlbumArtists => "Album Artists",
            Tab::Albums => "Albums",
            Tab::Genre => "Genre",
            Tab::Playlists => "Playlists",
            Tab::Search => "Search",
        }
    }

    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Self {
        Tab::ALL.get(i).copied().unwrap_or(Tab::Queue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    Library,
    Playlist,
    Lyrics,
}

impl FocusedPane {
    pub fn next(self) -> Self {
        match self {
            FocusedPane::Library => FocusedPane::Playlist,
            FocusedPane::Playlist => FocusedPane::Lyrics,
            FocusedPane::Lyrics => FocusedPane::Library,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FocusedPane::Library => FocusedPane::Lyrics,
            FocusedPane::Playlist => FocusedPane::Library,
            FocusedPane::Lyrics => FocusedPane::Playlist,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub state: PlayState,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            state: PlayState::Stopped,
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: 0.8,
            shuffle: false,
            repeat: RepeatMode::Off,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueState {
    pub tracks: Vec<usize>,
    pub current_index: Option<usize>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            selected_index: 0,
            scroll_offset: 0,
        }
    }
}
