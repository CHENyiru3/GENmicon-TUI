use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::palette;

use super::{ModalKind, ModalView, ViewAction, ViewEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellControlChoice {
    Background,
    Cancel,
}

impl ShellControlChoice {
    fn event(self) -> ViewEvent {
        match self {
            ShellControlChoice::Background => ViewEvent::ShellControlBackground,
            ShellControlChoice::Cancel => ViewEvent::ShellControlCancel,
        }
    }
}

pub struct ShellControlView {
    selected: ShellControlChoice,
}

impl ShellControlView {
    pub fn new() -> Self {
        Self {
            selected: ShellControlChoice::Background,
        }
    }

    fn toggle(&mut self) {
        self.selected = match self.selected {
            ShellControlChoice::Background => ShellControlChoice::Cancel,
            ShellControlChoice::Cancel => ShellControlChoice::Background,
        };
    }
}

impl ModalView for ShellControlView {
    fn kind(&self) -> ModalKind {
        ModalKind::ShellControl
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => ViewAction::Close,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.toggle();
                ViewAction::None
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                ViewAction::EmitAndClose(ViewEvent::ShellControlBackground)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                ViewAction::EmitAndClose(ViewEvent::ShellControlCancel)
            }
            KeyCode::Enter => ViewAction::EmitAndClose(self.selected.event()),
            _ => ViewAction::None,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::{
            style::Style,
            text::{Line, Span},
            widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
        };

        let popup_width = 62.min(area.width.saturating_sub(4));
        let popup_height = 11.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width - popup_width) / 2,
            y: (area.height - popup_height) / 2,
            width: popup_width,
            height: popup_height,
        };

        Clear.render(popup_area, buf);

        let option_line = |choice: ShellControlChoice, key: &'static str, label: &'static str| {
            let selected = self.selected == choice;
            let style = if selected {
                Style::default()
                    .fg(palette::SELECTION_TEXT)
                    .bg(palette::SELECTION_BG)
            } else {
                Style::default().fg(palette::TEXT_PRIMARY)
            };
            Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, style),
                Span::styled(format!("{key:<3}"), style.bold()),
                Span::styled(label, style),
            ])
        };

        let lines = vec![
            Line::from(Span::styled(
                "Foreground shell command is still running.",
                Style::default().fg(palette::TEXT_PRIMARY),
            )),
            Line::from(""),
            option_line(
                ShellControlChoice::Background,
                "B",
                "Background - detach and keep the command running",
            ),
            option_line(
                ShellControlChoice::Cancel,
                "C",
                "Cancel - stop the command and interrupt this turn",
            ),
        ];

        let view = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(Line::from(vec![Span::styled(
                        " Shell command ",
                        Style::default().fg(palette::DEEPSEEK_BLUE).bold(),
                    )]))
                    .title_bottom(Line::from(Span::styled(
                        " Enter select | Esc close ",
                        Style::default().fg(palette::TEXT_MUTED),
                    )))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette::BORDER_COLOR))
                    .style(Style::default().bg(palette::DEEPSEEK_INK))
                    .padding(Padding::uniform(1)),
            )
            .style(Style::default().fg(palette::TEXT_PRIMARY));

        view.render(popup_area, buf);
    }
}
