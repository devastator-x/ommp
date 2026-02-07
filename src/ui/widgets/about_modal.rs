use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme::Theme;

// Gradient colors for the logo: cyan -> blue -> magenta -> purple
const GRAD: [Color; 6] = [
    Color::Rgb(0, 255, 255),   // bright cyan
    Color::Rgb(0, 200, 255),   // cyan-blue
    Color::Rgb(80, 140, 255),  // blue
    Color::Rgb(160, 100, 255), // blue-purple
    Color::Rgb(200, 80, 255),  // purple
    Color::Rgb(255, 60, 220),  // magenta
];

const LOGO: [&str; 7] = [
    r"  ___  __  __ __  __ ____  ",
    r" / _ \|  \/  |  \/  |  _ \ ",
    r"| | | | \  / | \  / | |_) |",
    r"| | | | |\/| | |\/| |  __/ ",
    r"| |_| | |  | | |  | | |    ",
    r" \___/|_|  |_|_|  |_|_|    ",
    r"                            ",
];

pub fn render_about_modal(frame: &mut Frame, area: Rect, theme: &Theme) {
    let modal = centered_rect(55, 60, area);

    frame.render_widget(Clear, modal);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" About ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let mut lines: Vec<Line> = Vec::new();

    // Empty line for top padding
    lines.push(Line::from(""));

    // Render logo with horizontal gradient
    for (row_idx, row) in LOGO.iter().enumerate() {
        let spans: Vec<Span> = row
            .chars()
            .enumerate()
            .map(|(col, ch)| {
                // Gradient based on column position + slight row offset
                let progress = (col as f32 / row.len().max(1) as f32
                    + row_idx as f32 * 0.08)
                    .fract();
                let idx = (progress * (GRAD.len() - 1) as f32) as usize;
                let color = GRAD[idx.min(GRAD.len() - 1)];
                let style = Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD);
                Span::styled(String::from(ch), style)
            })
            .collect();
        lines.push(Line::from(spans));
    }

    // Subtitle
    lines.push(Line::from(Span::styled(
        "Oh My Music Player",
        Style::default()
            .fg(Color::Rgb(180, 140, 255))
            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
    )).alignment(Alignment::Center));

    lines.push(Line::from(""));

    // Divider
    let divider_w = inner.width.saturating_sub(4) as usize;
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(divider_w),
        Style::default().fg(Color::Indexed(240)),
    )).alignment(Alignment::Center));

    lines.push(Line::from(""));

    // Info rows
    let info: &[(&str, &str)] = &[
        ("Version", "0.1.0"),
        ("License", "MIT"),
    ];

    for (label, value) in info {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:12}", label),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                *value,
                Style::default().fg(theme.fg),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Links
    let link_style = Style::default()
        .fg(Color::Rgb(100, 180, 255))
        .add_modifier(Modifier::UNDERLINED);
    let label_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    lines.push(Line::from(vec![
        Span::styled("  \u{F09B} GitHub    ", label_style),
        Span::styled("github.com/devastator-x/ommp", link_style),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  \u{2665} Sponsor   ", label_style),
        Span::styled("github.com/sponsors/devastator-x", link_style),
    ]));

    lines.push(Line::from(""));

    // Music note decoration
    lines.push(Line::from(Span::styled(
        "\u{266B} \u{266A} \u{266B}  Terminal music, your way  \u{266B} \u{266A} \u{266B}",
        Style::default().fg(Color::Rgb(100, 200, 255)),
    )).alignment(Alignment::Center));

    lines.push(Line::from(""));

    // Footer
    lines.push(Line::from(Span::styled(
        "Press Esc to close",
        Style::default().fg(Color::DarkGray),
    )).alignment(Alignment::Center));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
