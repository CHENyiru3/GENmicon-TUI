//! Generic footer/status bar renderer.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::Renderable;

pub const WAVE_GLYPHS: [char; 8] = [
    '\u{2581}', // ▁
    '\u{2582}', // ▂
    '\u{2583}', // ▃
    '\u{2584}', // ▄
    '\u{2585}', // ▅
    '\u{2586}', // ▆
    '\u{2587}', // ▇
    '\u{2588}', // █
];

#[derive(Debug, Clone)]
pub struct FooterProps {
    pub model: String,
    pub mode_label: String,
    pub mode_color: Color,
    pub text_dim_color: Color,
    pub text_hint_color: Color,
    pub text_muted_color: Color,
    pub footer_bg: Color,
    pub working_strip_color: Color,
    pub state_label: String,
    pub state_color: Color,
    pub coherence: Vec<Span<'static>>,
    pub agents: Vec<Span<'static>>,
    pub reasoning_replay: Vec<Span<'static>>,
    pub cache: Vec<Span<'static>>,
    pub mcp: Vec<Span<'static>>,
    pub worked: Vec<Span<'static>>,
    pub cost: Vec<Span<'static>>,
    pub toast: Option<FooterToast>,
    pub banner: Option<FooterBanner>,
    pub working_strip_frame: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct FooterToast {
    pub text: String,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct FooterBanner {
    pub text: String,
    pub color: Color,
}

/// One frame of the footer's live-work wave animation.
#[must_use]
pub fn footer_working_strip_glyph_at(col: usize, width: usize, frame: u64) -> char {
    if width == 0 {
        return ' ';
    }

    let t = frame as f64 / 1000.0;
    let x = col as f64;

    let primary = (x * 0.52 - t * 8.0).sin();
    let swell = (x * 0.18 + t * 3.1).sin() * 0.35;
    let shimmer = (x * 1.35 - t * 11.0).sin() * 0.12;
    let value = ((primary + swell + shimmer) / 1.47).clamp(-1.0, 1.0);
    let normalized = (value + 1.0) * 0.5;
    let idx = (normalized * (WAVE_GLYPHS.len() - 1) as f64).round() as usize;
    WAVE_GLYPHS[idx.min(WAVE_GLYPHS.len() - 1)]
}

/// Build the per-frame live-work wave string of `width` characters.
#[must_use]
pub fn footer_working_strip_string(width: usize, frame: u64) -> String {
    let mut out = String::with_capacity(width * 4);
    for col in 0..width {
        out.push(footer_working_strip_glyph_at(col, width, frame));
    }
    out
}

/// Pure-render footer. Build once per frame, then `render(area, buf)`.
pub struct FooterWidget {
    props: FooterProps,
}

impl FooterWidget {
    #[must_use]
    pub fn new(props: FooterProps) -> Self {
        Self { props }
    }

    fn auxiliary_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        let parts: Vec<&Vec<Span<'static>>> = [
            &self.props.coherence,
            &self.props.agents,
            &self.props.reasoning_replay,
            &self.props.cache,
            &self.props.mcp,
            &self.props.worked,
        ]
        .into_iter()
        .filter(|spans| !spans.is_empty())
        .collect();

        for end in (0..=parts.len()).rev() {
            let mut combined: Vec<Span<'static>> = Vec::new();
            for (i, part) in parts[..end].iter().enumerate() {
                if i > 0 {
                    combined.push(Span::raw("  "));
                }
                combined.extend(part.iter().cloned());
            }
            if span_width(&combined) <= max_width {
                return combined;
            }
        }
        Vec::new()
    }

    fn alert_spans(text: &str, color: Color, max_width: usize) -> Vec<Span<'static>> {
        let truncated = truncate_to_width(text, max_width.max(1));
        vec![Span::styled(truncated, Style::default().fg(color))]
    }

    fn status_line_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        if max_width == 0 {
            return Vec::new();
        }

        let mode_label = self.props.mode_label.as_str();
        let sep = " \u{00B7} ";
        let model = self.props.model.as_str();
        let show_status = self.props.state_label != "ready";
        let status_label = self.props.state_label.as_str();
        let cost_text = spans_text(&self.props.cost);
        let show_cost = !cost_text.is_empty();

        let mode_w = mode_label.width();
        let sep_w = sep.width();
        let model_w = UnicodeWidthStr::width(model);
        let status_w = status_label.width();
        let cost_w = cost_text.width();

        let full_w = mode_w
            + sep_w
            + model_w
            + if show_cost { sep_w + cost_w } else { 0 }
            + if show_status { sep_w + status_w } else { 0 };
        if (show_cost || show_status) && full_w <= max_width {
            return self.build_status_line_spans(
                mode_label,
                model.to_string(),
                show_cost.then(|| cost_text.clone()),
                show_status.then_some(status_label),
            );
        }

        if show_cost {
            let with_cost_w = mode_w + sep_w + model_w + sep_w + cost_w;
            if with_cost_w <= max_width {
                return self.build_status_line_spans(
                    mode_label,
                    model.to_string(),
                    Some(cost_text.clone()),
                    None,
                );
            }
        }

        let mode_model_w = mode_w + sep_w + model_w;
        if mode_model_w <= max_width {
            return self.build_status_line_spans(mode_label, model.to_string(), None, None);
        }

        let prefix_w = mode_w + sep_w;
        if prefix_w < max_width {
            let model_budget = max_width - prefix_w;
            if model_budget >= 4 {
                let truncated = truncate_to_width(model, model_budget);
                if !truncated.is_empty() {
                    return self.build_status_line_spans(mode_label, truncated, None, None);
                }
            }
        }

        if mode_w <= max_width {
            return vec![Span::styled(
                mode_label.to_string(),
                Style::default().fg(self.props.mode_color),
            )];
        }
        vec![Span::styled(
            truncate_to_width(mode_label, max_width),
            Style::default().fg(self.props.mode_color),
        )]
    }

    fn build_status_line_spans(
        &self,
        mode_label: &str,
        model_label: String,
        cost: Option<String>,
        status: Option<&str>,
    ) -> Vec<Span<'static>> {
        let sep = " \u{00B7} ";
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !mode_label.is_empty() {
            spans.push(Span::styled(
                mode_label.to_string(),
                Style::default().fg(self.props.mode_color),
            ));
        }
        if !model_label.is_empty() {
            if !spans.is_empty() {
                spans.push(Span::styled(
                    sep.to_string(),
                    Style::default().fg(self.props.text_dim_color),
                ));
            }
            spans.push(Span::styled(
                model_label,
                Style::default().fg(self.props.text_hint_color),
            ));
        }
        if let Some(cost_text) = cost {
            if !spans.is_empty() {
                spans.push(Span::styled(
                    sep.to_string(),
                    Style::default().fg(self.props.text_dim_color),
                ));
            }
            spans.push(Span::styled(
                cost_text,
                Style::default().fg(self.props.text_muted_color),
            ));
        }
        if let Some(status_label) = status {
            if !spans.is_empty() {
                spans.push(Span::styled(
                    sep.to_string(),
                    Style::default().fg(self.props.text_dim_color),
                ));
            }
            spans.push(Span::styled(
                status_label.to_string(),
                Style::default().fg(self.props.state_color),
            ));
        }
        spans
    }
}

impl Renderable for FooterWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let available_width = area.width as usize;
        if available_width == 0 {
            return;
        }

        let right_spans = self.auxiliary_spans(available_width);
        let right_width = span_width(&right_spans);
        let min_gap = if right_width > 0 { 2 } else { 0 };
        let max_left_width = available_width
            .saturating_sub(right_width)
            .saturating_sub(min_gap)
            .max(1);

        let left_spans = if let Some(banner) = self.props.banner.as_ref() {
            Self::alert_spans(&banner.text, banner.color, max_left_width)
        } else if let Some(toast) = self.props.toast.as_ref() {
            Self::alert_spans(&toast.text, toast.color, max_left_width)
        } else {
            self.status_line_spans(max_left_width)
        };

        let left_width = span_width(&left_spans);
        let spacer_width = available_width.saturating_sub(left_width + right_width);

        let spacer_span = match self.props.working_strip_frame {
            Some(frame) if spacer_width > 0 => Span::styled(
                footer_working_strip_string(spacer_width, frame),
                Style::default().fg(self.props.working_strip_color),
            ),
            _ => Span::raw(" ".repeat(spacer_width)),
        };

        let mut all_spans = left_spans;
        all_spans.push(spacer_span);
        all_spans.extend(right_spans);

        let paragraph =
            Paragraph::new(Line::from(all_spans)).style(Style::default().bg(self.props.footer_bg));
        paragraph.render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}

fn spans_text(spans: &[Span<'_>]) -> String {
    spans.iter().map(|s| s.content.as_ref()).collect::<String>()
}

fn span_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|span| span.content.width()).sum()
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return text.chars().take(max_width).collect();
    }

    let mut out = String::new();
    let mut width = 0usize;
    let limit = max_width.saturating_sub(3);
    for ch in text.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > limit {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_props() -> FooterProps {
        FooterProps {
            model: "deepseek-v4-flash".to_string(),
            mode_label: "agent".to_string(),
            mode_color: Color::Cyan,
            text_dim_color: Color::DarkGray,
            text_hint_color: Color::Gray,
            text_muted_color: Color::DarkGray,
            footer_bg: Color::Black,
            working_strip_color: Color::Cyan,
            state_label: "ready".to_string(),
            state_color: Color::DarkGray,
            coherence: Vec::new(),
            agents: Vec::new(),
            reasoning_replay: Vec::new(),
            cache: Vec::new(),
            mcp: Vec::new(),
            worked: Vec::new(),
            cost: Vec::new(),
            toast: None,
            banner: None,
            working_strip_frame: None,
        }
    }

    fn render_at_width(props: FooterProps, width: u16) -> String {
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        FooterWidget::new(props).render(area, &mut buf);
        (0..area.width)
            .map(|x| buf[(x, 0)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    #[test]
    fn render_emits_mode_and_model_when_idle() {
        let line = render_at_width(base_props(), 60);

        assert!(line.contains("agent"));
        assert!(line.contains("deepseek-v4-flash"));
        assert!(!line.contains("ready"));
    }

    #[test]
    fn footer_priority_drops_status_before_model() {
        let mut props = base_props();
        props.state_label = "refreshing context".to_string();
        props.state_color = Color::Yellow;

        let line = render_at_width(props, 40);

        assert!(line.contains("agent"));
        assert!(line.contains("deepseek-v4-flash"));
        assert!(!line.contains("refreshing"));
    }

    #[test]
    fn toast_replaces_status_line() {
        let mut props = base_props();
        props.toast = Some(FooterToast {
            text: "session saved".to_string(),
            color: Color::Green,
        });

        let line = render_at_width(props, 60);

        assert!(line.contains("session saved"));
        assert!(!line.contains("agent"));
        assert!(!line.contains("deepseek-v4-flash"));
    }

    #[test]
    fn banner_outranks_toast() {
        let mut props = base_props();
        props.toast = Some(FooterToast {
            text: "session saved".to_string(),
            color: Color::Green,
        });
        props.banner = Some(FooterBanner {
            text: "retry 2 in 7s".to_string(),
            color: Color::Yellow,
        });

        let line = render_at_width(props, 60);

        assert!(line.contains("retry 2"));
        assert!(!line.contains("session saved"));
    }

    #[test]
    fn working_strip_string_width_matches_request() {
        for width in [0usize, 1, 8, 60, 200] {
            let s = footer_working_strip_string(width, 7);
            assert_eq!(s.chars().count(), width, "width {width} mismatch");
        }
    }
}
