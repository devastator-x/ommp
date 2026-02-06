use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone)]
pub struct LayoutAreas {
    pub status_bar: Rect,
    pub tab_bar: Rect,
    pub library: Rect,
    pub playlist: Rect,
    pub lyrics: Rect,
    pub progress_bar: Rect,
}

impl LayoutAreas {
    pub fn compute(area: Rect, pane_widths: [u16; 3]) -> Self {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // Status bar (bordered = 2 content + 2 border)
                Constraint::Length(3),  // Tab bar (bordered)
                Constraint::Min(10),   // Dashboard
                Constraint::Length(3), // Progress bar (bordered)
            ])
            .split(area);

        let status_bar = vertical[0];
        let tab_bar = vertical[1];
        let dashboard = vertical[2];
        let progress_bar = vertical[3];

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(pane_widths[0]),
                Constraint::Percentage(pane_widths[1]),
                Constraint::Percentage(pane_widths[2]),
            ])
            .split(dashboard);

        Self {
            status_bar,
            tab_bar,
            library: columns[0],
            playlist: columns[1],
            lyrics: columns[2],
            progress_bar,
        }
    }
}
