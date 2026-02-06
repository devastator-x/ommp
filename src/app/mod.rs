pub mod handler;
pub mod state;

use std::path::PathBuf;

use crate::audio::{AudioEngine, PlayerCommand};
use crate::library::Library;
use state::*;

#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    PlayTrack(usize),
    PauseResume,
    Stop,
    NextTrack,
    PrevTrack,
    SetVolume(f32),
    VolumeUp,
    VolumeDown,
    Seek(f64),
    SeekForward,
    SeekBackward,
    ToggleShuffle,
    CycleRepeat,
    SwitchTab(Tab),
    FocusNext,
    FocusPrev,
    FocusPane(FocusedPane),
    AddToQueue(Vec<usize>),
    ClearQueue,
    RemoveFromQueue(usize),
    PlayQueueIndex(usize),
    UpdatePosition { position_secs: f64, duration_secs: f64 },
    TrackFinished,
    SetQueueSelection(usize),
    SearchQuery(String),
    EnterSearchMode,
    ExitSearchMode,
}

pub struct App {
    pub should_quit: bool,
    pub tab: Tab,
    pub focus: FocusedPane,
    pub playback: PlaybackState,
    pub queue: QueueState,
    pub library: Library,
    pub music_dir: PathBuf,
    pub search_query: String,
    pub search_mode: bool,
    pub search_results: Vec<usize>,
    audio_engine: Option<AudioEngine>,
}

impl App {
    pub fn new(music_dir: PathBuf) -> Self {
        Self {
            should_quit: false,
            tab: Tab::Queue,
            focus: FocusedPane::Library,
            playback: PlaybackState::default(),
            queue: QueueState::default(),
            library: Library::new(),
            music_dir,
            search_query: String::new(),
            search_mode: false,
            search_results: Vec::new(),
            audio_engine: None,
        }
    }

    pub fn set_audio_engine(&mut self, engine: AudioEngine) {
        self.audio_engine = Some(engine);
    }

    pub fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => {
                self.should_quit = true;
                if let Some(ref engine) = self.audio_engine {
                    engine.send(PlayerCommand::Stop);
                }
            }
            AppAction::PlayTrack(track_idx) => {
                if track_idx < self.library.tracks.len() {
                    let path = self.library.tracks[track_idx].path.clone();
                    let dur = self.library.tracks[track_idx].duration.as_secs_f64();
                    if let Some(ref engine) = self.audio_engine {
                        engine.send(PlayerCommand::Play(path));
                    }
                    self.playback.state = PlayState::Playing;
                    self.playback.position_secs = 0.0;
                    self.playback.duration_secs = dur;
                }
            }
            AppAction::PauseResume => match self.playback.state {
                PlayState::Playing => {
                    if let Some(ref engine) = self.audio_engine {
                        engine.send(PlayerCommand::Pause);
                    }
                    self.playback.state = PlayState::Paused;
                }
                PlayState::Paused => {
                    if let Some(ref engine) = self.audio_engine {
                        engine.send(PlayerCommand::Resume);
                    }
                    self.playback.state = PlayState::Playing;
                }
                PlayState::Stopped => {
                    // Try to play current queue item
                    if let Some(idx) = self.queue.current_index {
                        if let Some(&track_idx) = self.queue.tracks.get(idx) {
                            self.handle_action(AppAction::PlayTrack(track_idx));
                        }
                    }
                }
            },
            AppAction::Stop => {
                if let Some(ref engine) = self.audio_engine {
                    engine.send(PlayerCommand::Stop);
                }
                self.playback.state = PlayState::Stopped;
                self.playback.position_secs = 0.0;
            }
            AppAction::NextTrack => {
                self.play_next();
            }
            AppAction::PrevTrack => {
                self.play_prev();
            }
            AppAction::SetVolume(vol) => {
                self.playback.volume = vol.clamp(0.0, 1.0);
                if let Some(ref engine) = self.audio_engine {
                    engine.send(PlayerCommand::SetVolume(self.playback.volume));
                }
            }
            AppAction::VolumeUp => {
                let vol = (self.playback.volume + 0.05).min(1.0);
                self.handle_action(AppAction::SetVolume(vol));
            }
            AppAction::VolumeDown => {
                let vol = (self.playback.volume - 0.05).max(0.0);
                self.handle_action(AppAction::SetVolume(vol));
            }
            AppAction::Seek(secs) => {
                let clamped = secs.clamp(0.0, self.playback.duration_secs);
                if let Some(ref engine) = self.audio_engine {
                    engine.send(PlayerCommand::Seek(clamped));
                }
                self.playback.position_secs = clamped;
            }
            AppAction::SeekForward => {
                let pos = self.playback.position_secs + 5.0;
                self.handle_action(AppAction::Seek(pos));
            }
            AppAction::SeekBackward => {
                let pos = self.playback.position_secs - 5.0;
                self.handle_action(AppAction::Seek(pos));
            }
            AppAction::ToggleShuffle => {
                self.playback.shuffle = !self.playback.shuffle;
            }
            AppAction::CycleRepeat => {
                self.playback.repeat = self.playback.repeat.next();
            }
            AppAction::SwitchTab(tab) => {
                self.tab = tab;
            }
            AppAction::FocusNext => {
                self.focus = self.focus.next();
            }
            AppAction::FocusPrev => {
                self.focus = self.focus.prev();
            }
            AppAction::FocusPane(pane) => {
                self.focus = pane;
            }
            AppAction::AddToQueue(track_indices) => {
                let was_empty = self.queue.tracks.is_empty();
                self.queue.tracks.extend(track_indices);
                if was_empty && !self.queue.tracks.is_empty() {
                    self.queue.current_index = Some(0);
                }
            }
            AppAction::ClearQueue => {
                self.queue.tracks.clear();
                self.queue.current_index = None;
                self.queue.selected_index = 0;
                self.queue.scroll_offset = 0;
            }
            AppAction::RemoveFromQueue(idx) => {
                if idx < self.queue.tracks.len() {
                    self.queue.tracks.remove(idx);
                    if self.queue.tracks.is_empty() {
                        self.queue.current_index = None;
                    } else if let Some(ref mut ci) = self.queue.current_index {
                        if idx < *ci {
                            *ci -= 1;
                        } else if idx == *ci && *ci >= self.queue.tracks.len() {
                            *ci = self.queue.tracks.len() - 1;
                        }
                    }
                }
            }
            AppAction::PlayQueueIndex(idx) => {
                if idx < self.queue.tracks.len() {
                    self.queue.current_index = Some(idx);
                    let track_idx = self.queue.tracks[idx];
                    self.handle_action(AppAction::PlayTrack(track_idx));
                }
            }
            AppAction::UpdatePosition { position_secs, duration_secs } => {
                self.playback.position_secs = position_secs;
                if duration_secs > 0.0 {
                    self.playback.duration_secs = duration_secs;
                }
            }
            AppAction::TrackFinished => {
                self.play_next();
            }
            AppAction::SetQueueSelection(idx) => {
                if idx < self.queue.tracks.len() {
                    self.queue.selected_index = idx;
                }
            }
            AppAction::SearchQuery(query) => {
                self.search_results = self.library.search(&query);
                self.search_query = query;
            }
            AppAction::EnterSearchMode => {
                self.search_mode = true;
            }
            AppAction::ExitSearchMode => {
                self.search_mode = false;
            }
        }
    }

    fn play_next(&mut self) {
        if self.queue.tracks.is_empty() {
            return;
        }

        match self.playback.repeat {
            RepeatMode::One => {
                if let Some(idx) = self.queue.current_index {
                    let track_idx = self.queue.tracks[idx];
                    let path = self.library.tracks[track_idx].path.clone();
                    let dur = self.library.tracks[track_idx].duration.as_secs_f64();
                    if let Some(ref engine) = self.audio_engine {
                        engine.send(PlayerCommand::Play(path));
                    }
                    self.playback.state = PlayState::Playing;
                    self.playback.position_secs = 0.0;
                    self.playback.duration_secs = dur;
                }
            }
            _ => {
                let next = if self.playback.shuffle {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    Some(rng.gen_range(0..self.queue.tracks.len()))
                } else if let Some(idx) = self.queue.current_index {
                    let next_idx = idx + 1;
                    if next_idx < self.queue.tracks.len() {
                        Some(next_idx)
                    } else if self.playback.repeat == RepeatMode::All {
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    Some(0)
                };

                if let Some(next_idx) = next {
                    self.queue.current_index = Some(next_idx);
                    let track_idx = self.queue.tracks[next_idx];
                    let path = self.library.tracks[track_idx].path.clone();
                    let dur = self.library.tracks[track_idx].duration.as_secs_f64();
                    if let Some(ref engine) = self.audio_engine {
                        engine.send(PlayerCommand::Play(path));
                    }
                    self.playback.state = PlayState::Playing;
                    self.playback.position_secs = 0.0;
                    self.playback.duration_secs = dur;
                } else {
                    self.playback.state = PlayState::Stopped;
                    self.playback.position_secs = 0.0;
                }
            }
        }
    }

    fn play_prev(&mut self) {
        if self.queue.tracks.is_empty() {
            return;
        }

        // If more than 3 seconds in, restart current track
        if self.playback.position_secs > 3.0 {
            if let Some(idx) = self.queue.current_index {
                let track_idx = self.queue.tracks[idx];
                let path = self.library.tracks[track_idx].path.clone();
                let dur = self.library.tracks[track_idx].duration.as_secs_f64();
                if let Some(ref engine) = self.audio_engine {
                    engine.send(PlayerCommand::Play(path));
                }
                self.playback.position_secs = 0.0;
                self.playback.duration_secs = dur;
                return;
            }
        }

        let prev = if let Some(idx) = self.queue.current_index {
            if idx > 0 {
                Some(idx - 1)
            } else if self.playback.repeat == RepeatMode::All {
                Some(self.queue.tracks.len() - 1)
            } else {
                Some(0)
            }
        } else {
            Some(0)
        };

        if let Some(prev_idx) = prev {
            self.queue.current_index = Some(prev_idx);
            let track_idx = self.queue.tracks[prev_idx];
            let path = self.library.tracks[track_idx].path.clone();
            let dur = self.library.tracks[track_idx].duration.as_secs_f64();
            if let Some(ref engine) = self.audio_engine {
                engine.send(PlayerCommand::Play(path));
            }
            self.playback.state = PlayState::Playing;
            self.playback.position_secs = 0.0;
            self.playback.duration_secs = dur;
        }
    }

    pub fn current_track(&self) -> Option<&crate::library::track::Track> {
        self.queue
            .current_index
            .and_then(|qi| self.queue.tracks.get(qi))
            .and_then(|&ti| self.library.tracks.get(ti))
    }
}
