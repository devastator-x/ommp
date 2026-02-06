pub mod input;

#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Tick,
    Audio(AudioEvent),
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
