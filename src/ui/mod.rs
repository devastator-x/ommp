pub mod layout;
pub mod pane;
pub mod panes;
pub mod theme;
pub mod widgets;

use ratatui::Frame;

use crate::app::App;
use crate::app::state::{FocusedPane, Tab};
use layout::LayoutAreas;
use pane::Pane;
use panes::album_artists_pane::AlbumArtistsPane;
use panes::albums_pane::AlbumsPane;
use panes::artists_pane::ArtistsPane;
use panes::dir_browser_pane::DirBrowserPane;
use panes::genre_pane::GenrePane;
use panes::library_pane::LibraryPane;
use panes::lyrics_pane::LyricsPane;
use panes::playlists_pane::PlaylistsPane;
use panes::queue_pane::QueuePane;
use panes::search_pane::SearchPane;
use theme::Theme;
use widgets::progress_bar;
use widgets::status_bar;
use widgets::tab_bar;

pub struct Ui {
    pub theme: Theme,
    pub library_pane: LibraryPane,
    pub dir_browser_pane: DirBrowserPane,
    pub queue_pane: QueuePane,
    pub artists_pane: ArtistsPane,
    pub album_artists_pane: AlbumArtistsPane,
    pub albums_pane: AlbumsPane,
    pub genre_pane: GenrePane,
    pub playlists_pane: PlaylistsPane,
    pub search_pane: SearchPane,
    pub lyrics_pane: LyricsPane,
}

impl Ui {
    pub fn new(music_dir: std::path::PathBuf) -> Self {
        Self {
            theme: Theme::default(),
            library_pane: LibraryPane::new(),
            dir_browser_pane: DirBrowserPane::new(music_dir),
            queue_pane: QueuePane::new(),
            artists_pane: ArtistsPane::new(),
            album_artists_pane: AlbumArtistsPane::new(),
            albums_pane: AlbumsPane::new(),
            genre_pane: GenrePane::new(),
            playlists_pane: PlaylistsPane::new(),
            search_pane: SearchPane::new(),
            lyrics_pane: LyricsPane::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        let areas = LayoutAreas::compute(frame.area());

        // Status bar
        status_bar::render_status_bar(frame, areas.status_bar, app, &self.theme);

        // Tab bar
        tab_bar::render_tab_bar(frame, areas.tab_bar, app.tab, &self.theme);

        // Left pane (varies by tab)
        let lib_focused = app.focus == FocusedPane::Library;
        match app.tab {
            Tab::Queue => self.library_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Directories => self.dir_browser_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Artists => self.artists_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::AlbumArtists => self.album_artists_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Albums => self.albums_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Genre => self.genre_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Playlists => self.playlists_pane.render(frame, areas.library, lib_focused, app, &self.theme),
            Tab::Search => self.search_pane.render(frame, areas.library, lib_focused, app, &self.theme),
        }

        // Center pane (Queue)
        let playlist_focused = app.focus == FocusedPane::Playlist;
        self.queue_pane.render(frame, areas.playlist, playlist_focused, app, &self.theme);

        // Right pane (Lyrics)
        let lyrics_focused = app.focus == FocusedPane::Lyrics;
        self.lyrics_pane.render(frame, areas.lyrics, lyrics_focused, app, &self.theme);

        // Progress bar
        progress_bar::render_progress_bar(frame, areas.progress_bar, app, &self.theme);
    }

    pub fn refresh_dir_browser(&mut self, app: &App) {
        self.dir_browser_pane.refresh(app);
    }
}
