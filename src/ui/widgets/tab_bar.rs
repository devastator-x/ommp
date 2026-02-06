use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::Tab;
use crate::ui::theme::Theme;

pub fn render_tab_bar(frame: &mut Frame, area: Rect, current: Tab, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_unfocused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build tab spans manually for center alignment
    let mut spans = Vec::new();
    for (i, tab) in Tab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", theme.dim_style));
        }
        let style = if i == current.index() {
            theme.tab_active
        } else {
            theme.tab_inactive
        };
        spans.push(Span::styled(tab.title(), style));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

/// Returns which tab index was clicked given mouse x position
pub fn tab_hit_test(area: Rect, mouse_x: u16) -> Option<usize> {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);

    if mouse_x < inner.x || mouse_x >= inner.x + inner.width {
        return None;
    }

    // Calculate total content width to find center offset
    let divider_len = 3; // " │ "
    let mut total_width: usize = 0;
    for (i, tab) in Tab::ALL.iter().enumerate() {
        if i > 0 {
            total_width += divider_len;
        }
        total_width += tab.title().len();
    }

    let inner_w = inner.width as usize;
    let pad_left = if inner_w > total_width {
        (inner_w - total_width) / 2
    } else {
        0
    };

    let rel_x = (mouse_x - inner.x) as usize;
    if rel_x < pad_left {
        return None;
    }

    let content_x = rel_x - pad_left;
    let mut pos = 0;
    for (i, tab) in Tab::ALL.iter().enumerate() {
        if i > 0 {
            pos += divider_len;
        }
        let title_len = tab.title().len();
        if content_x < pos + title_len {
            return Some(i);
        }
        pos += title_len;
    }
    None
}
