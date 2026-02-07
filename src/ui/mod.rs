pub mod layout;
pub mod pane;
pub mod panes;
pub mod theme;
pub mod widgets;

use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use widgets::{help_modal, playlist_modal, search_modal};
use widgets::playlist_modal::PlaylistModalMode;

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
    pub last_click: Option<(std::time::Instant, u16, u16)>,
    /// Last known mouse position (column, row) for hover tracking
    pub mouse_pos: Option<(u16, u16)>,
    /// Tab index currently hovered by mouse
    pub hovered_tab: Option<usize>,
    /// Pane width percentages [Library, Playlist, Lyrics], sum = 100
    pub pane_widths: [u16; 3],
    /// Resize mode active (Ctrl+E)
    pub resize_mode: bool,
    /// Border being dragged: 0 = lib|playlist, 1 = playlist|lyrics, None = not dragging
    pub dragging_border: Option<u8>,
    /// Ctrl+E pressed, waiting for next key
    pub chord_pending: bool,
    /// Help modal visible
    pub show_help_modal: bool,
    /// Search modal visible
    pub show_search_modal: bool,
    /// Search modal input text
    pub search_modal_input: String,
    /// Search modal filtered results (track indices)
    pub search_modal_results: Vec<usize>,
    /// Search modal selected result index
    pub search_modal_selected: usize,
    /// Search modal scroll offset
    pub search_modal_scroll: usize,
    /// Playlist modal visible ("b" key)
    pub show_playlist_modal: bool,
    /// Playlist modal selected index
    pub playlist_modal_selected: usize,
    /// Playlist modal mode (List / Create / Rename)
    pub playlist_modal_mode: PlaylistModalMode,
    /// Playlist modal text input (for create/rename)
    pub playlist_modal_input: String,
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
            last_click: None,
            mouse_pos: None,
            hovered_tab: None,
            pane_widths: [20, 60, 20],
            resize_mode: false,
            dragging_border: None,
            chord_pending: false,
            show_help_modal: false,
            show_search_modal: false,
            search_modal_input: String::new(),
            search_modal_results: Vec::new(),
            search_modal_selected: 0,
            search_modal_scroll: 0,
            show_playlist_modal: false,
            playlist_modal_selected: 0,
            playlist_modal_mode: PlaylistModalMode::List,
            playlist_modal_input: String::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        let areas = LayoutAreas::compute(frame.area(), self.pane_widths);

        // Status bar
        status_bar::render_status_bar(frame, areas.status_bar, app, &self.theme, self.resize_mode);

        // Tab bar
        tab_bar::render_tab_bar(frame, areas.tab_bar, app.tab, self.hovered_tab, &self.theme);

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

        // Resize mode: overlay yellow border on focused pane
        if self.resize_mode {
            let focused_area = match app.focus {
                FocusedPane::Library => areas.library,
                FocusedPane::Playlist => areas.playlist,
                FocusedPane::Lyrics => areas.lyrics,
            };
            let overlay = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            frame.render_widget(overlay, focused_area);
        }

        // Modal overlays (rendered last, on top of everything)
        if self.show_search_modal {
            search_modal::render_search_modal(
                frame,
                frame.area(),
                &self.search_modal_input,
                &self.search_modal_results,
                self.search_modal_selected,
                self.search_modal_scroll,
                app,
                &self.theme,
            );
        }

        if self.show_help_modal {
            help_modal::render_help_modal(frame, frame.area(), &self.theme);
        }

        if self.show_playlist_modal {
            playlist_modal::render_playlist_modal(
                frame,
                frame.area(),
                self.playlist_modal_selected,
                &self.playlist_modal_mode,
                &self.playlist_modal_input,
                app,
                &self.theme,
            );
        }
    }

    pub fn refresh_dir_browser(&mut self, app: &App) {
        self.dir_browser_pane.refresh(app);
    }
}
