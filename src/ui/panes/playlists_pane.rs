use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, AppAction};
use crate::ui::pane::Pane;
use crate::ui::theme::Theme;

pub struct PlaylistsPane;

impl PlaylistsPane {
    pub fn new() -> Self {
        Self
    }
}

impl Pane for PlaylistsPane {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, _app: &App, theme: &Theme) {
        let border_color = if focused {
            theme.border_focused
        } else {
            theme.border_unfocused
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Playlists ")
            .title_style(Style::default().fg(if focused {
                theme.border_focused
            } else {
                theme.fg
            }));

        let content = Paragraph::new("No saved playlists")
            .style(theme.dim_style)
            .block(block);

        frame.render_widget(content, area);
    }

    fn handle_key(&mut self, _key: KeyEvent, _app: &App) -> Option<AppAction> {
        None
    }

    fn handle_mouse(&mut self, _event: MouseEvent, _area: Rect, _app: &App) -> Option<AppAction> {
        None
    }
}
