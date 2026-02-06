use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

use crate::app::state::{FocusedPane, Tab};
use crate::app::{App, AppAction};
use crate::ui::layout::LayoutAreas;
use crate::ui::pane::Pane;
use crate::ui::widgets::{progress_bar, tab_bar};
use crate::ui::Ui;

pub fn handle_key_event(key: KeyEvent, app: &App, ui: &mut Ui) -> Vec<AppAction> {
    let mut actions = Vec::new();

    // In search input mode, route everything to search pane
    if app.search_mode {
        if let Some(action) = ui.search_pane.handle_key(key, app) {
            actions.push(action);
        }
        return actions;
    }

    // Global keybindings first
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => {
            actions.push(AppAction::Quit);
            return actions;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            actions.push(AppAction::Quit);
            return actions;
        }
        (_, KeyCode::Char(' ')) => {
            actions.push(AppAction::PauseResume);
            return actions;
        }
        (_, KeyCode::Char('n')) => {
            actions.push(AppAction::NextTrack);
            return actions;
        }
        (KeyModifiers::SHIFT, KeyCode::Char('N')) => {
            actions.push(AppAction::PrevTrack);
            return actions;
        }
        (_, KeyCode::Char('+')) | (_, KeyCode::Char('=')) => {
            actions.push(AppAction::VolumeUp);
            return actions;
        }
        (_, KeyCode::Char('-')) => {
            actions.push(AppAction::VolumeDown);
            return actions;
        }
        (_, KeyCode::Right) => {
            actions.push(AppAction::SeekForward);
            return actions;
        }
        (_, KeyCode::Left) => {
            actions.push(AppAction::SeekBackward);
            return actions;
        }
        (_, KeyCode::Char('s')) => {
            actions.push(AppAction::ToggleShuffle);
            return actions;
        }
        (_, KeyCode::Char('r')) => {
            actions.push(AppAction::CycleRepeat);
            return actions;
        }
        (_, KeyCode::Tab) => {
            actions.push(AppAction::FocusNext);
            return actions;
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            actions.push(AppAction::FocusPrev);
            return actions;
        }
        // Tab switching with number keys
        (_, KeyCode::Char('1')) => {
            actions.push(AppAction::SwitchTab(Tab::Queue));
            return actions;
        }
        (_, KeyCode::Char('2')) => {
            actions.push(AppAction::SwitchTab(Tab::Directories));
            return actions;
        }
        (_, KeyCode::Char('3')) => {
            actions.push(AppAction::SwitchTab(Tab::Artists));
            return actions;
        }
        (_, KeyCode::Char('4')) => {
            actions.push(AppAction::SwitchTab(Tab::AlbumArtists));
            return actions;
        }
        (_, KeyCode::Char('5')) => {
            actions.push(AppAction::SwitchTab(Tab::Albums));
            return actions;
        }
        (_, KeyCode::Char('6')) => {
            actions.push(AppAction::SwitchTab(Tab::Genre));
            return actions;
        }
        (_, KeyCode::Char('7')) => {
            actions.push(AppAction::SwitchTab(Tab::Playlists));
            return actions;
        }
        (_, KeyCode::Char('8')) => {
            actions.push(AppAction::SwitchTab(Tab::Search));
            return actions;
        }
        // h/l for pane focus
        (_, KeyCode::Char('h')) => {
            actions.push(AppAction::FocusPrev);
            return actions;
        }
        (_, KeyCode::Char('l')) => {
            actions.push(AppAction::FocusNext);
            return actions;
        }
        _ => {}
    }

    // Route to focused pane
    let action = match app.focus {
        FocusedPane::Library => match app.tab {
            Tab::Queue => ui.library_pane.handle_key(key, app),
            Tab::Directories => ui.dir_browser_pane.handle_key(key, app),
            Tab::Artists => ui.artists_pane.handle_key(key, app),
            Tab::AlbumArtists => ui.album_artists_pane.handle_key(key, app),
            Tab::Albums => ui.albums_pane.handle_key(key, app),
            Tab::Genre => ui.genre_pane.handle_key(key, app),
            Tab::Playlists => ui.playlists_pane.handle_key(key, app),
            Tab::Search => ui.search_pane.handle_key(key, app),
        },
        FocusedPane::Playlist => {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => None,
                KeyCode::Char('k') | KeyCode::Up => None,
                _ => ui.queue_pane.handle_key(key, app),
            }
        }
        FocusedPane::Lyrics => ui.lyrics_pane.handle_key(key, app),
    };

    // Auto-focus to Queue pane when adding tracks from Library
    if let Some(ref a) = action {
        if matches!(a, AppAction::AddToQueue(_)) && app.focus == FocusedPane::Library {
            actions.push(AppAction::FocusPane(FocusedPane::Playlist));
        }
    }

    if let Some(a) = action {
        actions.push(a);
    }

    actions
}

pub fn handle_mouse_event(
    mouse: MouseEvent,
    app: &App,
    ui: &mut Ui,
    terminal_area: ratatui::layout::Rect,
) -> Vec<AppAction> {
    let mut actions = Vec::new();
    let areas = LayoutAreas::compute(terminal_area);

    let x = mouse.column;
    let y = mouse.row;

    // Store mouse position for hover tracking across all event types
    ui.mouse_pos = Some((x, y));

    // Determine which pane the mouse is in
    let in_library = x >= areas.library.x
        && x < areas.library.x + areas.library.width
        && y >= areas.library.y
        && y < areas.library.y + areas.library.height;

    let in_playlist = x >= areas.playlist.x
        && x < areas.playlist.x + areas.playlist.width
        && y >= areas.playlist.y
        && y < areas.playlist.y + areas.playlist.height;

    let in_lyrics = x >= areas.lyrics.x
        && x < areas.lyrics.x + areas.lyrics.width
        && y >= areas.lyrics.y
        && y < areas.lyrics.y + areas.lyrics.height;

    // --- Hover tracking (runs on every mouse event including Moved) ---
    update_hover(ui, &areas, app, x, y, in_library, in_playlist);

    // --- Tab hover highlight ---
    if y >= areas.tab_bar.y && y < areas.tab_bar.y + areas.tab_bar.height {
        ui.hovered_tab = tab_bar::tab_hit_test(areas.tab_bar, x);
    } else {
        ui.hovered_tab = None;
    }

    // --- Focus switching on hover (any mouse event in a pane) ---
    if in_library && app.focus != FocusedPane::Library {
        actions.push(AppAction::FocusPane(FocusedPane::Library));
    } else if in_playlist && app.focus != FocusedPane::Playlist {
        actions.push(AppAction::FocusPane(FocusedPane::Playlist));
    } else if in_lyrics && app.focus != FocusedPane::Lyrics {
        actions.push(AppAction::FocusPane(FocusedPane::Lyrics));
    }

    // --- Handle specific event kinds ---
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Double-click detection
            let is_double_click = if let Some((last_time, _last_col, last_row)) = ui.last_click {
                last_time.elapsed() < Duration::from_millis(400) && last_row == y
            } else {
                false
            };
            ui.last_click = Some((Instant::now(), x, y));

            // Tab bar click
            if y >= areas.tab_bar.y && y < areas.tab_bar.y + areas.tab_bar.height {
                if let Some(tab_idx) = tab_bar::tab_hit_test(areas.tab_bar, x) {
                    actions.push(AppAction::SwitchTab(Tab::from_index(tab_idx)));
                }
                return actions;
            }

            // Progress bar click
            if y >= areas.progress_bar.y && y < areas.progress_bar.y + areas.progress_bar.height {
                let gauge_area = progress_bar::progress_gauge_area(areas.progress_bar);
                if x >= gauge_area.x && x < gauge_area.x + gauge_area.width {
                    let ratio = (x - gauge_area.x) as f64 / gauge_area.width as f64;
                    let seek_pos = ratio * app.playback.duration_secs;
                    actions.push(AppAction::Seek(seek_pos));
                }
                return actions;
            }

            // Double-click in playlist → play that track
            if is_double_click && in_playlist {
                let block = ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL);
                let inner = block.inner(areas.playlist);
                if y >= inner.y && y < inner.y + inner.height {
                    let clicked = ui.queue_pane.scroll_offset + (y - inner.y) as usize;
                    if clicked < app.queue.tracks.len() {
                        actions.push(AppAction::PlayQueueIndex(clicked));
                        return actions;
                    }
                }
            }

            // Double-click in library → activate (Enter)
            if is_double_click && in_library {
                let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
                let dbl_action = match app.tab {
                    Tab::Queue => ui.library_pane.handle_key(enter_key, app),
                    Tab::Directories => ui.dir_browser_pane.handle_key(enter_key, app),
                    Tab::Artists => ui.artists_pane.handle_key(enter_key, app),
                    Tab::AlbumArtists => ui.album_artists_pane.handle_key(enter_key, app),
                    Tab::Albums => ui.albums_pane.handle_key(enter_key, app),
                    Tab::Genre => ui.genre_pane.handle_key(enter_key, app),
                    Tab::Playlists => ui.playlists_pane.handle_key(enter_key, app),
                    Tab::Search => ui.search_pane.handle_key(enter_key, app),
                };
                if let Some(action) = dbl_action {
                    if matches!(action, AppAction::AddToQueue(_)) {
                        actions.push(AppAction::FocusPane(FocusedPane::Playlist));
                    }
                    actions.push(action);
                    return actions;
                }
            }

            // Single click — route to pane for selection
            if in_library {
                let action = match app.tab {
                    Tab::Queue => ui.library_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Directories => ui.dir_browser_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Artists => ui.artists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::AlbumArtists => ui.album_artists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Albums => ui.albums_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Genre => ui.genre_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Playlists => ui.playlists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Search => ui.search_pane.handle_mouse(mouse, areas.library, app),
                };
                if let Some(a) = action {
                    actions.push(a);
                }
            } else if in_playlist {
                if let Some(a) = ui.queue_pane.handle_mouse(mouse, areas.playlist, app) {
                    actions.push(a);
                }
                // Update queue selection on click
                let block = ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL);
                let inner = block.inner(areas.playlist);
                if y >= inner.y && y < inner.y + inner.height {
                    let clicked = ui.queue_pane.scroll_offset + (y - inner.y) as usize;
                    if clicked < app.queue.tracks.len() {
                        actions.push(AppAction::SetQueueSelection(clicked));
                    }
                }
            } else if in_lyrics {
                if let Some(a) = ui.lyrics_pane.handle_mouse(mouse, areas.lyrics, app) {
                    actions.push(a);
                }
            }
        }
        MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
            if in_library {
                let action = match app.tab {
                    Tab::Queue => ui.library_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Directories => ui.dir_browser_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Artists => ui.artists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::AlbumArtists => ui.album_artists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Albums => ui.albums_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Genre => ui.genre_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Playlists => ui.playlists_pane.handle_mouse(mouse, areas.library, app),
                    Tab::Search => ui.search_pane.handle_mouse(mouse, areas.library, app),
                };
                if let Some(a) = action {
                    actions.push(a);
                }
            } else if in_playlist {
                if let Some(a) = ui.queue_pane.handle_mouse(mouse, areas.playlist, app) {
                    actions.push(a);
                }
            } else if in_lyrics {
                if let Some(a) = ui.lyrics_pane.handle_mouse(mouse, areas.lyrics, app) {
                    actions.push(a);
                }
            }
        }
        _ => {
            // Moved and other events — hover already handled above
        }
    }

    actions
}

/// Clear all hover_row state across all panes
fn clear_all_hovers(ui: &mut Ui) {
    ui.queue_pane.hover_row = None;
    ui.library_pane.hover_row = None;
    ui.dir_browser_pane.hover_row = None;
    ui.artists_pane.hover_row = None;
    ui.album_artists_pane.hover_row = None;
    ui.albums_pane.hover_row = None;
    ui.genre_pane.hover_row = None;
    ui.search_pane.hover_row = None;
}

/// Update hover_row state for panes based on mouse position
fn update_hover(
    ui: &mut Ui,
    areas: &LayoutAreas,
    app: &App,
    x: u16,
    y: u16,
    in_library: bool,
    in_playlist: bool,
) {
    clear_all_hovers(ui);

    if in_playlist {
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL);
        let inner = block.inner(areas.playlist);
        if x >= inner.x && x < inner.x + inner.width
            && y >= inner.y && y < inner.y + inner.height
        {
            let row = ui.queue_pane.scroll_offset + (y - inner.y) as usize;
            if row < app.queue.tracks.len() {
                ui.queue_pane.hover_row = Some(row);
            }
        }
    } else if in_library {
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL);
        let inner = block.inner(areas.library);
        if x >= inner.x && x < inner.x + inner.width
            && y >= inner.y && y < inner.y + inner.height
        {
            let visual_row = (y - inner.y) as usize;
            match app.tab {
                Tab::Queue => {
                    let row = ui.library_pane.scroll_offset + visual_row;
                    ui.library_pane.hover_row = Some(row);
                }
                Tab::Directories => {
                    let row = ui.dir_browser_pane.scroll_offset + visual_row;
                    ui.dir_browser_pane.hover_row = Some(row);
                }
                Tab::Artists => {
                    let row = ui.artists_pane.scroll_offset + visual_row;
                    ui.artists_pane.hover_row = Some(row);
                }
                Tab::AlbumArtists => {
                    let row = ui.album_artists_pane.scroll_offset + visual_row;
                    ui.album_artists_pane.hover_row = Some(row);
                }
                Tab::Albums => {
                    let row = ui.albums_pane.scroll_offset + visual_row;
                    ui.albums_pane.hover_row = Some(row);
                }
                Tab::Genre => {
                    let row = ui.genre_pane.scroll_offset + visual_row;
                    ui.genre_pane.hover_row = Some(row);
                }
                Tab::Playlists => {
                    // PlaylistsPane has no list to hover
                }
                Tab::Search => {
                    let row = ui.search_pane.scroll_offset + visual_row;
                    ui.search_pane.hover_row = Some(row);
                }
            }
        }
    }
}

/// Refresh hover state from the stored mouse position.
/// Call this on Tick events so hover stays updated even without Moved events.
/// Returns a focus action if the mouse is over a different pane.
pub fn refresh_hover(app: &App, ui: &mut Ui, terminal_area: ratatui::layout::Rect) -> Vec<AppAction> {
    let mut actions = Vec::new();
    if let Some((x, y)) = ui.mouse_pos {
        let areas = LayoutAreas::compute(terminal_area);
        let in_library = x >= areas.library.x
            && x < areas.library.x + areas.library.width
            && y >= areas.library.y
            && y < areas.library.y + areas.library.height;
        let in_playlist = x >= areas.playlist.x
            && x < areas.playlist.x + areas.playlist.width
            && y >= areas.playlist.y
            && y < areas.playlist.y + areas.playlist.height;
        let in_lyrics = x >= areas.lyrics.x
            && x < areas.lyrics.x + areas.lyrics.width
            && y >= areas.lyrics.y
            && y < areas.lyrics.y + areas.lyrics.height;
        update_hover(ui, &areas, app, x, y, in_library, in_playlist);

        // Tab hover highlight
        if y >= areas.tab_bar.y && y < areas.tab_bar.y + areas.tab_bar.height {
            ui.hovered_tab = tab_bar::tab_hit_test(areas.tab_bar, x);
        } else {
            ui.hovered_tab = None;
        }

        // Focus switching on hover
        if in_library && app.focus != FocusedPane::Library {
            actions.push(AppAction::FocusPane(FocusedPane::Library));
        } else if in_playlist && app.focus != FocusedPane::Playlist {
            actions.push(AppAction::FocusPane(FocusedPane::Playlist));
        } else if in_lyrics && app.focus != FocusedPane::Lyrics {
            actions.push(AppAction::FocusPane(FocusedPane::Lyrics));
        }
    }
    actions
}

/// Update queue selection based on keyboard in playlist focus
pub fn update_queue_selection(app: &mut App, key: KeyEvent) {
    let count = app.queue.tracks.len();
    if count == 0 {
        return;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.queue.selected_index < count - 1 {
                app.queue.selected_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.queue.selected_index > 0 {
                app.queue.selected_index -= 1;
            }
        }
        _ => {}
    }
}
