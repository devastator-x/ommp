use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::app::state::PlayState;
use crate::ui::theme::Theme;

pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_unfocused))
        .title(" Status ")
        .title_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(inner);

    // Left: Play state + time
    let state_icon = match app.playback.state {
        PlayState::Playing => "▶",
        PlayState::Paused => "⏸",
        PlayState::Stopped => "⏹",
    };

    let pos = format_time(app.playback.position_secs);
    let dur = format_time(app.playback.duration_secs);

    let bitrate = app
        .current_track()
        .and_then(|t| t.bitrate)
        .map(|b| format!(" ({}kbps)", b))
        .unwrap_or_default();

    let left_line1 = Line::from(vec![
        Span::styled(
            format!(" {} {}", state_icon, match app.playback.state {
                PlayState::Playing => "Playing",
                PlayState::Paused => "Paused",
                PlayState::Stopped => "Stopped",
            }),
            Style::default().fg(theme.playing_indicator).add_modifier(Modifier::BOLD),
        ),
    ]);

    let left_line2 = Line::from(vec![
        Span::styled(
            format!(" {}/{}{}", pos, dur, bitrate),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let left = Paragraph::new(vec![left_line1, left_line2]);
    frame.render_widget(left, cols[0]);

    // Center: Track info
    let (title, artist_album) = if let Some(track) = app.current_track() {
        (
            track.title.clone(),
            format!("{} - {}", track.display_artist(), track.display_album()),
        )
    } else {
        ("No track playing".to_string(), String::new())
    };

    let center_line1 = Line::from(Span::styled(
        title,
        theme.title_style,
    )).alignment(Alignment::Center);

    let center_line2 = Line::from(Span::styled(
        artist_album,
        theme.artist_style,
    )).alignment(Alignment::Center);

    let center = Paragraph::new(vec![center_line1, center_line2]);
    frame.render_widget(center, cols[1]);

    // Right: Volume + shuffle/repeat
    let vol_pct = (app.playback.volume * 100.0) as u8;
    let shuffle_icon = if app.playback.shuffle { "🔀" } else { "⇥" };
    let repeat_icon = app.playback.repeat.symbol();

    let right_line1 = Line::from(Span::styled(
        format!("Vol: {}% ", vol_pct),
        Style::default().fg(Color::White),
    )).alignment(Alignment::Right);

    let right_line2 = Line::from(Span::styled(
        format!("{} {} ", shuffle_icon, repeat_icon),
        Style::default().fg(Color::Yellow),
    )).alignment(Alignment::Right);

    let right = Paragraph::new(vec![right_line1, right_line2]);
    frame.render_widget(right, cols[2]);
}

fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{}:{:02}", m, s)
}
