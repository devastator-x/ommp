use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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
            // Queue pane handles j/k for selection
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.queue.selected_index < app.queue.tracks.len().saturating_sub(1) {
                        // We need a special action for this
                        // For now, handle it inline
                        None // Handled below
                    } else {
                        None
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => None,
                _ => ui.queue_pane.handle_key(key, app),
            }
        }
        FocusedPane::Lyrics => ui.lyrics_pane.handle_key(key, app),
    };

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

    // Check tab bar clicks
    if y >= areas.tab_bar.y && y < areas.tab_bar.y + areas.tab_bar.height {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if let Some(tab_idx) = tab_bar::tab_hit_test(areas.tab_bar, x) {
                actions.push(AppAction::SwitchTab(Tab::from_index(tab_idx)));
            }
        }
        return actions;
    }

    // Check progress bar clicks
    if y >= areas.progress_bar.y && y < areas.progress_bar.y + areas.progress_bar.height {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let gauge_area = progress_bar::progress_gauge_area(areas.progress_bar);
            if x >= gauge_area.x && x < gauge_area.x + gauge_area.width {
                let ratio = (x - gauge_area.x) as f64 / gauge_area.width as f64;
                let seek_pos = ratio * app.playback.duration_secs;
                actions.push(AppAction::Seek(seek_pos));
            }
        }
        return actions;
    }

    // Check dashboard pane clicks
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

    // Focus on click
    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        if in_library && app.focus != FocusedPane::Library {
            actions.push(AppAction::FocusPane(FocusedPane::Library));
        } else if in_playlist && app.focus != FocusedPane::Playlist {
            actions.push(AppAction::FocusPane(FocusedPane::Playlist));
        } else if in_lyrics && app.focus != FocusedPane::Lyrics {
            actions.push(AppAction::FocusPane(FocusedPane::Lyrics));
        }
    }

    // Route mouse to focused pane's area
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
        // Handle queue pane mouse
        if let Some(a) = ui.queue_pane.handle_mouse(mouse, areas.playlist, app) {
            actions.push(a);
        }
        // Also handle selection update for queue
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL);
            let inner = block.inner(areas.playlist);
            if mouse.row >= inner.y && mouse.row < inner.y + inner.height {
                let clicked = ui.queue_pane.scroll_offset + (mouse.row - inner.y) as usize;
                if clicked < app.queue.tracks.len() {
                    // We'll set selected_index via a direct mutation after actions are applied
                    // For now, just note the click happened
                }
            }
        }
    } else if in_lyrics {
        if let Some(a) = ui.lyrics_pane.handle_mouse(mouse, areas.lyrics, app) {
            actions.push(a);
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
