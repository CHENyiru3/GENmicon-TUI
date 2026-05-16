use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::art::{ArtFrame, fit_rect_to_ratio, scale_art_lines};

#[derive(Debug, Clone)]
pub struct PanelChrome<'a> {
    pub title: Option<Line<'a>>,
    pub style: Style,
    pub border_style: Style,
    pub padding: Padding,
}

impl<'a> PanelChrome<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            style: Style::default(),
            border_style: Style::default(),
            padding: Padding::ZERO,
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    #[must_use]
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    #[must_use]
    pub fn inner(&self, area: Rect) -> Rect {
        self.block().inner(area)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.block().render(area, buf);
    }

    fn block(&self) -> Block<'a> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .style(self.style)
            .border_style(self.border_style)
            .padding(self.padding);
        if let Some(title) = self.title.clone() {
            block = block.title(title);
        }
        block
    }
}

impl Default for PanelChrome<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextPanelStyles {
    pub background: Color,
    pub border: Color,
    pub focused_border: Color,
    pub title: Color,
    pub focused_title: Color,
    pub text: Color,
    pub muted: Color,
}

#[derive(Debug, Clone)]
pub struct TextPanelProps<'a> {
    pub title: &'a str,
    pub lines: &'a [String],
    pub scroll: usize,
    pub focused: bool,
    pub empty_message: &'a str,
    pub styles: TextPanelStyles,
}

#[derive(Debug, Clone)]
pub struct ArtPanelProps<'a> {
    pub title: &'a str,
    pub art: Option<&'a ArtFrame>,
    pub fallback_lines: &'a [String],
    pub empty_message: &'a str,
    pub styles: TextPanelStyles,
}

pub fn render_text_panel(props: TextPanelProps<'_>, area: Rect, buf: &mut Buffer) {
    let block = panel_block(props.title, props.styles, props.focused);
    let inner = bordered_panel_inner(area);
    block.render(area, buf);
    render_lines(
        props.lines.to_vec(),
        props.scroll,
        inner,
        buf,
        props.styles,
        props.empty_message,
    );
}

pub fn render_art_or_text_panel(props: ArtPanelProps<'_>, area: Rect, buf: &mut Buffer) {
    let block = panel_block(props.title, props.styles, false);
    let inner = block.inner(area);
    block.render(area, buf);

    if let Some(frame) = props.art {
        let fitted = fit_rect_to_ratio(inner, frame.ratio_cols, frame.ratio_rows);
        if fitted.width > 0 && fitted.height > 0 {
            let scaled = scale_art_lines(&frame.lines, fitted.width, fitted.height);
            if !scaled.is_empty() {
                let art_height = u16::try_from(scaled.len()).unwrap_or(fitted.height);
                let y_offset = fitted.height.saturating_sub(art_height).saturating_div(2);
                let lines = scaled
                    .into_iter()
                    .map(|cells| {
                        Line::from(
                            cells
                                .into_iter()
                                .map(|cell| Span::styled(cell.symbol.to_string(), cell.style))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                let art_area = Rect {
                    x: fitted.x,
                    y: fitted.y.saturating_add(y_offset),
                    width: fitted.width,
                    height: art_height.min(fitted.height),
                };
                Paragraph::new(lines)
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().bg(props.styles.background))
                    .render(art_area, buf);
                return;
            }

            let art_area = Rect {
                x: fitted.x,
                y: fitted.y,
                width: fitted.width,
                height: fitted.height,
            };
            render_lines(
                props.fallback_lines.to_vec(),
                0,
                art_area,
                buf,
                props.styles,
                props.empty_message,
            );
            return;
        }
    }

    render_lines(
        props.fallback_lines.to_vec(),
        0,
        inner,
        buf,
        props.styles,
        props.empty_message,
    );
}

#[must_use]
pub fn bordered_panel_inner(area: Rect) -> Rect {
    Block::default().borders(Borders::ALL).inner(area)
}

#[must_use]
pub fn max_scroll_for_lines(lines: &[String], area: Rect) -> usize {
    let total_visual_lines = visual_line_count(lines, area.width);
    total_visual_lines.saturating_sub(usize::from(area.height))
}

#[must_use]
pub fn visual_line_count(lines: &[String], width: u16) -> usize {
    if lines.is_empty() {
        1
    } else {
        lines
            .iter()
            .map(|line| wrapped_line_count(line, width))
            .sum()
    }
}

fn render_lines(
    lines: Vec<String>,
    scroll: usize,
    area: Rect,
    buf: &mut Buffer,
    styles: TextPanelStyles,
    empty_message: &str,
) {
    let total_visual_lines = visual_line_count(&lines, area.width);
    let content = if lines.is_empty() {
        vec![Line::from(Span::styled(
            empty_message.to_string(),
            Style::default().fg(styles.muted),
        ))]
    } else {
        lines
            .into_iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(styles.text))))
            .collect()
    };
    let max_scroll = total_visual_lines.saturating_sub(usize::from(area.height));
    let scroll = scroll.min(max_scroll);
    Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .scroll((scroll.min(usize::from(u16::MAX)) as u16, 0))
        .style(Style::default().bg(styles.background))
        .render(area, buf);
}

fn wrapped_line_count(line: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    let line_width = UnicodeWidthStr::width(line);
    line_width
        .saturating_add(width - 1)
        .saturating_div(width)
        .max(1)
}

fn panel_block(title: &str, styles: TextPanelStyles, focused: bool) -> Block<'static> {
    let border = if focused {
        styles.focused_border
    } else {
        styles.border
    };
    Block::default()
        .title(Line::from(Span::styled(
            format!(" {title} "),
            Style::default().fg(if focused {
                styles.focused_title
            } else {
                styles.title
            }),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(styles.background))
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, style::Color};

    use super::*;

    fn test_styles() -> TextPanelStyles {
        TextPanelStyles {
            background: Color::Black,
            border: Color::DarkGray,
            focused_border: Color::Cyan,
            title: Color::Gray,
            focused_title: Color::White,
            text: Color::White,
            muted: Color::DarkGray,
        }
    }

    #[test]
    fn panel_renders_border_and_title() {
        let area = Rect::new(0, 0, 12, 3);
        let mut buf = Buffer::empty(area);

        PanelChrome::new().title("Menu").render(area, &mut buf);

        let top: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(top.contains("Menu"), "{top}");
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        assert_eq!(buf[(11, 2)].symbol(), "┘");
        assert_eq!(PanelChrome::new().inner(area), Rect::new(1, 1, 10, 1));
    }

    #[test]
    fn text_panel_renders_border_title_and_empty_message() {
        let area = Rect::new(0, 0, 24, 5);
        let mut buf = Buffer::empty(area);
        render_text_panel(
            TextPanelProps {
                title: "Status",
                lines: &[],
                scroll: 0,
                focused: false,
                empty_message: "Empty",
                styles: test_styles(),
            },
            area,
            &mut buf,
        );

        let top: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        let body: String = (0..area.width).map(|x| buf[(x, 1)].symbol()).collect();

        assert!(top.contains("Status"), "{top}");
        assert!(body.contains("Empty"), "{body}");
    }

    #[test]
    fn max_scroll_for_wrapped_lines_reports_overflow() {
        let lines = vec!["1234567890".to_string(), "abcdef".to_string()];

        assert_eq!(max_scroll_for_lines(&lines, Rect::new(0, 0, 5, 2)), 2);
    }
}
