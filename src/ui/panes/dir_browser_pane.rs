use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use std::path::PathBuf;

use crate::app::{App, AppAction};
use crate::ui::pane::Pane;
use crate::ui::theme::Theme;

pub struct DirBrowserPane {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub enum DirEntry {
    ParentDir,
    Directory(String),
    Track(usize),
}

impl DirBrowserPane {
    pub fn new(music_dir: PathBuf) -> Self {
        Self {
            current_dir: music_dir,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    pub fn refresh(&mut self, app: &App) {
        self.entries.clear();

        if self.current_dir != app.music_dir {
            self.entries.push(DirEntry::ParentDir);
        }

        let (subdirs, tracks) = app.library.get_directory_entries(&self.current_dir);
        for dir in subdirs {
            self.entries.push(DirEntry::Directory(dir));
        }
        for track_idx in tracks {
            self.entries.push(DirEntry::Track(track_idx));
        }
    }
}

impl Pane for DirBrowserPane {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, app: &App, theme: &Theme) {
        let border_color = if focused {
            theme.border_focused
        } else {
            theme.border_unfocused
        };

        let dir_name = self
            .current_dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", dir_name))
            .title_style(Style::default().fg(if focused {
                theme.border_focused
            } else {
                theme.fg
            }));

        let inner_height = block.inner(area).height as usize;

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(inner_height)
            .map(|(i, entry)| {
                let is_selected = i == self.selected;
                let style = if is_selected && focused {
                    Style::default()
                        .bg(theme.highlight_bg)
                        .fg(theme.highlight_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };

                let text = match entry {
                    DirEntry::ParentDir => "📁 ..".to_string(),
                    DirEntry::Directory(name) => format!("📁 {}", name),
                    DirEntry::Track(idx) => {
                        let t = &app.library.tracks[*idx];
                        format!("🎵 {}", t.title)
                    }
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn handle_key(&mut self, key: KeyEvent, app: &App) -> Option<AppAction> {
        let count = self.entries.len();
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
                match &self.entries[self.selected] {
                    DirEntry::ParentDir => {
                        if let Some(parent) = self.current_dir.parent() {
                            self.current_dir = parent.to_path_buf();
                            self.selected = 0;
                            self.scroll_offset = 0;
                            self.refresh(app);
                        }
                    }
                    DirEntry::Directory(name) => {
                        self.current_dir = self.current_dir.join(name);
                        self.selected = 0;
                        self.scroll_offset = 0;
                        self.refresh(app);
                    }
                    DirEntry::Track(idx) => {
                        // Add this track to queue and play it
                        return Some(AppAction::AddToQueue(vec![*idx]));
                    }
                }
                None
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.selected = 0;
                    self.scroll_offset = 0;
                    self.refresh(app);
                }
                None
            }
            _ => None,
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent, area: Rect, app: &App) -> Option<AppAction> {
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let count = self.entries.len();

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
            // Double-click not available in crossterm 0.28 - use single click + Enter
            MouseEventKind::ScrollDown => self.handle_scroll(false, app),
            MouseEventKind::ScrollUp => self.handle_scroll(true, app),
            _ => None,
        }
    }

    fn handle_scroll(&mut self, up: bool, _app: &App) -> Option<AppAction> {
        let count = self.entries.len();
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
