use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreTheme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub selected_bg: Color,
}

impl Default for CoreTheme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            surface: Color::Reset,
            border: Color::DarkGray,
            text: Color::White,
            muted: Color::Gray,
            accent: Color::Cyan,
            selected_bg: Color::Rgb(32, 45, 60),
        }
    }
}

impl CoreTheme {
    #[must_use]
    pub fn text_style(self) -> Style {
        Style::default().fg(self.text).bg(self.background)
    }

    #[must_use]
    pub fn muted_style(self) -> Style {
        Style::default().fg(self.muted).bg(self.background)
    }

    #[must_use]
    pub fn selected_style(self) -> Style {
        Style::default().fg(self.text).bg(self.selected_bg)
    }
}
