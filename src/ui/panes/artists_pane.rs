use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::{App, AppAction};
use crate::ui::pane::Pane;
use crate::ui::theme::Theme;

pub struct ArtistsPane {
    pub selected: usize,
    pub scroll_offset: usize,
}

impl ArtistsPane {
    pub fn new() -> Self {
        Self {
            selected: 0,
            scroll_offset: 0,
        }
    }
}

impl Pane for ArtistsPane {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, app: &App, theme: &Theme) {
        let artists = app.library.get_artists();
        let border_color = if focused {
            theme.border_focused
        } else {
            theme.border_unfocused
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" Artists ({}) ", artists.len()))
            .title_style(Style::default().fg(if focused {
                theme.border_focused
            } else {
                theme.fg
            }));

        let inner_height = block.inner(area).height as usize;

        let items: Vec<ListItem> = artists
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(inner_height)
            .map(|(i, artist)| {
                let is_selected = i == self.selected;
                let style = if is_selected && focused {
                    Style::default()
                        .bg(theme.highlight_bg)
                        .fg(theme.highlight_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };
                ListItem::new(Line::from(Span::styled(artist.as_str(), style)))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn handle_key(&mut self, key: KeyEvent, app: &App) -> Option<AppAction> {
        let artists = app.library.get_artists();
        let count = artists.len();
        if count == 0 {
            return None;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < count - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Enter => {
                if self.selected < count {
                    let tracks = app.library.get_tracks_by_artist(&artists[self.selected]);
                    if !tracks.is_empty() {
                        return Some(AppAction::AddToQueue(tracks));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent, area: Rect, app: &App) -> Option<AppAction> {
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let count = app.library.get_artists().len();

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if event.column >= inner.x
                    && event.column < inner.x + inner.width
                    && event.row >= inner.y
                    && event.row < inner.y + inner.height
                {
                    let clicked = self.scroll_offset + (event.row - inner.y) as usize;
                    if clicked < count {
                        self.selected = clicked;
                    }
                }
                None
            }
            MouseEventKind::ScrollDown => self.handle_scroll(false, app),
            MouseEventKind::ScrollUp => self.handle_scroll(true, app),
            _ => None,
        }
    }

    fn handle_scroll(&mut self, up: bool, app: &App) -> Option<AppAction> {
        let count = app.library.get_artists().len();
        if count == 0 {
            return None;
        }
        if up {
            self.scroll_offset = self.scroll_offset.saturating_sub(3);
        } else {
            self.scroll_offset = (self.scroll_offset + 3).min(count.saturating_sub(1));
        }
        None
    }
}
