mod app;
mod audio;
mod event;
mod library;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::handler;
use app::state::FocusedPane;
use app::App;
use audio::AudioEngine;
use event::input;
use event::{AudioEvent, Event};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let music_dir = dirs_music_path();

    // Event channel
    let (event_tx, event_rx) = crossbeam_channel::unbounded();

    // Spawn input thread
    let _input_handle = input::spawn_input_thread(event_tx.clone());
    let _tick_handle = input::spawn_tick_thread(event_tx.clone(), Duration::from_millis(200));

    // Audio engine
    let audio_engine = AudioEngine::new(event_tx.clone())?;

    // App state
    let mut app = App::new(music_dir.clone());
    app.set_audio_engine(audio_engine);

    // Scan library in background
    let scan_dir = music_dir.clone();
    let scan_handle = std::thread::spawn(move || {
        library::Library::scan(&scan_dir)
    });

    // UI
    let mut ui = ui::Ui::new(music_dir);

    // Initial render
    terminal.draw(|frame| {
        ui.render(frame, &app);
    })?;

    // Wait for library scan to complete (non-blocking check in event loop)
    let mut scan_done = false;
    let mut scan_join = Some(scan_handle);

    loop {
        // Check if library scan is done
        if !scan_done {
            if let Some(ref handle) = scan_join {
                if handle.is_finished() {
                    if let Some(handle) = scan_join.take() {
                        match handle.join() {
                            Ok(lib) => {
                                app.library = lib;
                                ui.refresh_dir_browser(&app);
                                scan_done = true;
                            }
                            Err(_) => {
                                scan_done = true;
                            }
                        }
                    }
                }
            }
        }

        // Process events
        match event_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                let actions = match event {
                    Event::Key(key) => {
                        // Handle queue selection directly for playlist focus
                        if app.focus == FocusedPane::Playlist && !app.search_mode {
                            handler::update_queue_selection(&mut app, key);
                        }
                        handler::handle_key_event(key, &app, &mut ui)
                    }
                    Event::Mouse(mouse) => {
                        let size = terminal.size()?;
                        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                        handler::handle_mouse_event(mouse, &app, &mut ui, area)
                    }
                    Event::Resize(_, _) => {
                        vec![] // Will re-render on next loop
                    }
                    Event::Tick => {
                        vec![] // Just re-render
                    }
                    Event::Audio(audio_event) => {
                        match audio_event {
                            AudioEvent::PositionUpdate {
                                position_secs,
                                duration_secs,
                            } => vec![app::AppAction::UpdatePosition {
                                position_secs,
                                duration_secs,
                            }],
                            AudioEvent::TrackFinished => vec![app::AppAction::TrackFinished],
                            AudioEvent::TrackError(_) => {
                                // Skip to next track on decode error
                                vec![app::AppAction::NextTrack]
                            }
                            AudioEvent::Playing => {
                                app.playback.state = app::state::PlayState::Playing;
                                vec![]
                            }
                            AudioEvent::Paused => {
                                app.playback.state = app::state::PlayState::Paused;
                                vec![]
                            }
                            AudioEvent::Stopped => {
                                app.playback.state = app::state::PlayState::Stopped;
                                vec![]
                            }
                        }
                    }
                };

                for action in actions {
                    app.handle_action(action);
                }

                if app.should_quit {
                    break;
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Just re-render
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        // Render
        terminal.draw(|frame| {
            ui.render(frame, &app);
        })?;
    }

    Ok(())
}

fn dirs_music_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let music = PathBuf::from(home).join("Music");
        if music.is_dir() {
            return music;
        }
    }
    PathBuf::from(".")
}
