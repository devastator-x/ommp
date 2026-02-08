pub mod input;

#[derive(Debug)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Tick,
    Audio(AudioEvent),
    LibraryReady(crate::library::Library),
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    PositionUpdate { position_secs: f64, duration_secs: f64 },
    TrackFinished,
    TrackError(String),
    Playing,
    Paused,
    Stopped,
}
