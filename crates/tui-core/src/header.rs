use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::Renderable;
use crate::text::truncate_to_width;

const CONTEXT_WARNING_THRESHOLD_PERCENT: f64 = 85.0;
const CONTEXT_CRITICAL_THRESHOLD_PERCENT: f64 = 95.0;
const CONTEXT_SIGNAL_WIDTH: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct HeaderMode<'a> {
    pub label: &'a str,
    pub fallback: &'a str,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct HeaderColors {
    pub background: Color,
    pub text_hint: Color,
    pub text_soft: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub accent: Color,
    pub status_warning: Color,
    pub status_error: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct HeaderProps<'a> {
    pub model: &'a str,
    pub workspace_name: &'a str,
    pub mode: HeaderMode<'a>,
    pub is_streaming: bool,
    pub context_window: Option<u32>,
    pub last_prompt_tokens: Option<u32>,
    pub reasoning_effort_label: Option<&'a str>,
    pub provider_label: Option<&'a str>,
    pub colors: HeaderColors,
}

pub struct HeaderWidget<'a> {
    props: HeaderProps<'a>,
}

impl<'a> HeaderWidget<'a> {
    #[must_use]
    pub fn new(props: HeaderProps<'a>) -> Self {
        Self { props }
    }

    fn span_width(spans: &[Span<'_>]) -> usize {
        spans.iter().map(|span| span.content.width()).sum()
    }

    fn context_percent(&self) -> Option<f64> {
        let used = f64::from(self.props.last_prompt_tokens?);
        let max = f64::from(self.props.context_window?);
        if max <= 0.0 {
            return None;
        }
        Some((used / max * 100.0).clamp(0.0, 100.0))
    }

    fn context_color(&self, percent: f64) -> Color {
        if percent >= CONTEXT_CRITICAL_THRESHOLD_PERCENT {
            self.props.colors.status_error
        } else if percent >= CONTEXT_WARNING_THRESHOLD_PERCENT {
            self.props.colors.status_warning
        } else {
            self.props.colors.accent
        }
    }

    fn context_signal_spans(&self, show_percent: bool) -> Vec<Span<'static>> {
        let Some(percent) = self.context_percent() else {
            return Vec::new();
        };

        let color = self.context_color(percent);
        let filled = ((percent / 100.0) * CONTEXT_SIGNAL_WIDTH as f64)
            .ceil()
            .clamp(0.0, CONTEXT_SIGNAL_WIDTH as f64) as usize;
        let empty = CONTEXT_SIGNAL_WIDTH.saturating_sub(filled);

        let mut spans = Vec::new();
        if show_percent {
            spans.push(Span::styled(
                format!("{percent:.0}%"),
                Style::default().fg(color),
            ));
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled("▰".repeat(filled), Style::default().fg(color)));
        spans.push(Span::styled(
            "▱".repeat(empty),
            Style::default().fg(self.props.colors.border),
        ));
        spans
    }

    fn context_percent_spans(&self) -> Vec<Span<'static>> {
        let Some(percent) = self.context_percent() else {
            return Vec::new();
        };

        vec![Span::styled(
            format!("{percent:.0}%"),
            Style::default().fg(self.context_color(percent)),
        )]
    }

    fn provider_chip_spans(&self) -> Vec<Span<'static>> {
        let Some(label) = self.props.provider_label else {
            return Vec::new();
        };
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        vec![Span::styled(
            trimmed.to_string(),
            Style::default()
                .fg(self.props.colors.accent)
                .add_modifier(Modifier::BOLD),
        )]
    }

    fn effort_chip_spans(&self, include_prefix: bool) -> Vec<Span<'static>> {
        let Some(label) = self.props.reasoning_effort_label else {
            return Vec::new();
        };
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        let is_off = trimmed.eq_ignore_ascii_case("off");
        let color = if is_off {
            self.props.colors.text_hint
        } else {
            self.props.colors.accent
        };
        let body = if !include_prefix {
            trimmed.to_string()
        } else if trimmed.eq_ignore_ascii_case("max") || trimmed.eq_ignore_ascii_case("maximum") {
            format!("\u{1F433} {trimmed}")
        } else {
            format!("\u{00B7} {trimmed}")
        };
        vec![Span::styled(body, Style::default().fg(color))]
    }

    fn status_variant(
        &self,
        show_stream_label: bool,
        show_percent: bool,
        show_signal: bool,
    ) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        let provider_spans = self.provider_chip_spans();
        let has_provider = !provider_spans.is_empty();
        if has_provider {
            spans.extend(provider_spans);
        }

        let effort_spans = self.effort_chip_spans(true);
        let has_effort = !effort_spans.is_empty();
        if has_effort {
            if has_provider {
                spans.push(Span::raw("  "));
            }
            spans.extend(effort_spans);
        }

        if self.props.is_streaming {
            if has_effort || has_provider {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                "●",
                Style::default()
                    .fg(self.props.colors.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            if show_stream_label {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    "Live",
                    Style::default().fg(self.props.colors.text_soft),
                ));
            }
        }

        let context_spans = if show_signal {
            self.context_signal_spans(show_percent)
        } else if show_percent {
            self.context_percent_spans()
        } else {
            Vec::new()
        };
        if !context_spans.is_empty() {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.extend(context_spans);
        }

        spans
    }

    fn right_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        let candidates = [
            self.status_variant(true, true, true),
            self.status_variant(false, true, true),
            self.status_variant(false, true, false),
            self.status_variant(false, false, true),
        ];

        candidates
            .into_iter()
            .find(|spans| Self::span_width(spans) <= max_width)
            .unwrap_or_default()
    }

    fn metadata_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        let workspace = self.props.workspace_name.trim();
        let model = self.props.model.trim();

        if max_width < 4 || (workspace.is_empty() && model.is_empty()) {
            return Vec::new();
        }

        if workspace.is_empty() {
            return vec![Span::styled(
                truncate_to_width(model, max_width),
                Style::default().fg(self.props.colors.text_hint),
            )];
        }

        if model.is_empty() || max_width < 12 {
            return vec![Span::styled(
                truncate_to_width(workspace, max_width),
                Style::default().fg(self.props.colors.text_secondary),
            )];
        }

        let separator_width = 3; // " · "
        if workspace.width() + separator_width + model.width() <= max_width {
            return vec![
                Span::styled(
                    workspace.to_string(),
                    Style::default().fg(self.props.colors.text_secondary),
                ),
                Span::styled(" · ", Style::default().fg(self.props.colors.text_hint)),
                Span::styled(
                    model.to_string(),
                    Style::default().fg(self.props.colors.text_hint),
                ),
            ];
        }

        let content_width = max_width.saturating_sub(separator_width);
        if content_width < 9 {
            return vec![Span::styled(
                truncate_to_width(workspace, max_width),
                Style::default().fg(self.props.colors.text_secondary),
            )];
        }

        let workspace_width = workspace.width();
        let model_width = model.width();
        let total_width = workspace_width + model_width;
        let min_workspace = 4;
        let min_model = 4;

        let proportional_workspace =
            ((content_width as f64 * workspace_width as f64) / total_width as f64).round() as usize;
        let workspace_budget =
            proportional_workspace.clamp(min_workspace, content_width.saturating_sub(min_model));
        let model_budget = content_width.saturating_sub(workspace_budget);

        vec![
            Span::styled(
                truncate_to_width(workspace, workspace_budget),
                Style::default().fg(self.props.colors.text_secondary),
            ),
            Span::styled(" · ", Style::default().fg(self.props.colors.text_hint)),
            Span::styled(
                truncate_to_width(model, model_budget),
                Style::default().fg(self.props.colors.text_hint),
            ),
        ]
    }

    fn left_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        if max_width == 0 {
            return Vec::new();
        }

        let mode_style = Style::default()
            .fg(self.props.mode.color)
            .add_modifier(Modifier::BOLD);

        if max_width < self.props.mode.label.width() {
            return vec![Span::styled(
                self.props.mode.fallback.to_string(),
                mode_style,
            )];
        }

        let mut spans = vec![Span::styled(self.props.mode.label.to_string(), mode_style)];
        let metadata_width = max_width
            .saturating_sub(self.props.mode.label.width())
            .saturating_sub(2);
        let metadata = if metadata_width >= 4 {
            self.metadata_spans(metadata_width)
        } else {
            Vec::new()
        };

        if !metadata.is_empty() {
            spans.push(Span::raw("  "));
            spans.extend(metadata);
        }

        spans
    }
}

impl Renderable for HeaderWidget<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let available = area.width as usize;
        let right_budget = available.saturating_sub(6);
        let right_spans = self.right_spans(right_budget);
        let right_width = Self::span_width(&right_spans);
        let spacer_min = usize::from(right_width > 0);
        let left_budget = available.saturating_sub(right_width + spacer_min);
        let left_spans = self.left_spans(left_budget);
        let left_width = Self::span_width(&left_spans);
        let spacer_width = available.saturating_sub(left_width + right_width);

        let mut spans = left_spans;
        if spacer_width > 0 {
            spans.push(Span::raw(" ".repeat(spacer_width)));
        }
        spans.extend(right_spans);

        let line = Line::from(spans);
        let paragraph =
            Paragraph::new(line).style(Style::default().bg(self.props.colors.background));
        paragraph.render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, style::Color};

    use super::*;

    fn props<'a>(
        width_model: &'a str,
        workspace_name: &'a str,
        is_streaming: bool,
    ) -> HeaderProps<'a> {
        HeaderProps {
            model: width_model,
            workspace_name,
            mode: HeaderMode {
                label: "Agent",
                fallback: "a",
                color: Color::Green,
            },
            is_streaming,
            context_window: None,
            last_prompt_tokens: None,
            reasoning_effort_label: None,
            provider_label: None,
            colors: HeaderColors {
                background: Color::Black,
                text_hint: Color::Gray,
                text_soft: Color::White,
                text_secondary: Color::Gray,
                border: Color::DarkGray,
                accent: Color::Cyan,
                status_warning: Color::Yellow,
                status_error: Color::Red,
            },
        }
    }

    fn render_header(props: HeaderProps<'_>, width: u16) -> String {
        let widget = HeaderWidget::new(props);
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        (0..width).map(|x| buf[(x, 0)].symbol()).collect::<String>()
    }

    #[test]
    fn header_renders_mode_and_metadata() {
        let rendered = render_header(props("model", "workspace", false), 40);

        assert!(rendered.contains("Agent"));
        assert!(rendered.contains("workspace"));
        assert!(rendered.contains("model"));
    }

    #[test]
    fn header_preserves_context_signal_when_narrow() {
        let mut props = props("", "", true);
        props.context_window = Some(100);
        props.last_prompt_tokens = Some(88);

        let rendered = render_header(props, 12);

        assert!(rendered.contains('%') || rendered.contains('▰'));
    }
}
