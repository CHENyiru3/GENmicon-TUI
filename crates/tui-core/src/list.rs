use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::text::truncate_to_width;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRow<'a> {
    pub label: Cow<'a, str>,
    pub detail: Option<Cow<'a, str>>,
    pub enabled: bool,
}

impl<'a> ListRow<'a> {
    #[must_use]
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            enabled: true,
        }
    }

    #[must_use]
    pub fn detail(mut self, detail: impl Into<Cow<'a, str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListStyles {
    pub normal: Style,
    pub selected: Style,
    pub disabled: Style,
}

impl Default for ListStyles {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            selected: Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            disabled: Style::default().fg(Color::DarkGray),
        }
    }
}

pub fn render_list_rows(
    area: Rect,
    buf: &mut Buffer,
    rows: &[ListRow<'_>],
    selected: Option<usize>,
    styles: ListStyles,
) {
    for (idx, row) in rows.iter().take(area.height as usize).enumerate() {
        let y = area.y + idx as u16;
        let style = if !row.enabled {
            styles.disabled
        } else if Some(idx) == selected {
            styles.selected
        } else {
            styles.normal
        };
        let prefix = if Some(idx) == selected { "> " } else { "  " };
        let mut content = format!("{prefix}{}", row.label);
        if let Some(detail) = &row.detail {
            content.push_str("  ");
            content.push_str(detail);
        }
        let content = truncate_to_width(&content, area.width as usize);
        Paragraph::new(Line::from(Span::styled(content, style)))
            .render(Rect::new(area.x, y, area.width, 1), buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect};

    use super::*;

    #[test]
    fn render_list_rows_marks_selected_row() {
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = Buffer::empty(area);
        let rows = [
            ListRow::new("first"),
            ListRow::new("second").detail("detail"),
            ListRow::new("third").disabled(),
        ];

        render_list_rows(area, &mut buf, &rows, Some(1), ListStyles::default());

        let selected: String = (0..area.width).map(|x| buf[(x, 1)].symbol()).collect();
        assert!(selected.starts_with("> second"), "{selected}");
        let selected_style = buf[(0, 1)].style();
        assert_eq!(selected_style.fg, Some(Color::White));
        assert_eq!(selected_style.bg, Some(Color::DarkGray));
        assert!(selected_style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(buf[(0, 2)].style().fg, Some(Color::DarkGray));
    }
}
