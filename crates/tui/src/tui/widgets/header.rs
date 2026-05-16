//! Header bar widget displaying mode, workspace/model context, and session status.

use deepseek_tui_core::header::{
    HeaderColors, HeaderMode, HeaderProps, HeaderWidget as CoreHeaderWidget,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::palette;
use crate::tui::app::AppMode;

use super::Renderable;

/// Data required to render the header bar.
pub struct HeaderData<'a> {
    pub model: &'a str,
    pub workspace_name: &'a str,
    pub mode: AppMode,
    pub is_streaming: bool,
    pub background: Color,
    /// Total tokens used in this session (cumulative, for display).
    pub total_tokens: u32,
    /// Context window size for the model (if known).
    pub context_window: Option<u32>,
    /// Accumulated session cost in the active display currency.
    pub session_cost: f64,
    /// Active context input tokens used for context utilization. Callers should
    /// pass a sanitized live-context estimate, not cumulative API usage.
    pub last_prompt_tokens: Option<u32>,
    /// Short label for the current reasoning-effort tier (e.g. "max", "high",
    /// "off"). Rendered as a chip when space allows.
    pub reasoning_effort_label: Option<&'a str>,
    /// Short label for the active provider (e.g. "NIM"). When `None` (the
    /// default-DeepSeek case), no provider chip is rendered. Surfaces the
    /// fact that requests are going somewhere other than DeepSeek's API so
    /// it's visible at a glance after a `/provider nvidia-nim`.
    pub provider_label: Option<&'a str>,
}

impl<'a> HeaderData<'a> {
    /// Create header data from common app fields.
    #[must_use]
    pub fn new(
        mode: AppMode,
        model: &'a str,
        workspace_name: &'a str,
        is_streaming: bool,
        background: Color,
    ) -> Self {
        Self {
            model,
            workspace_name,
            mode,
            is_streaming,
            background,
            total_tokens: 0,
            context_window: None,
            session_cost: 0.0,
            last_prompt_tokens: None,
            reasoning_effort_label: None,
            provider_label: None,
        }
    }

    /// Attach a short reasoning-effort label for the header chip.
    #[must_use]
    pub fn with_reasoning_effort(mut self, label: Option<&'a str>) -> Self {
        self.reasoning_effort_label = label;
        self
    }

    /// Attach a short provider label for the header chip. Pass `None` when on
    /// the default DeepSeek provider so the chip is hidden.
    #[must_use]
    pub fn with_provider(mut self, label: Option<&'a str>) -> Self {
        self.provider_label = label;
        self
    }

    /// Set token/cost fields.
    #[must_use]
    pub fn with_usage(
        mut self,
        total_tokens: u32,
        context_window: Option<u32>,
        session_cost: f64,
        active_context_input_tokens: Option<u32>,
    ) -> Self {
        self.total_tokens = total_tokens;
        self.context_window = context_window;
        self.session_cost = session_cost;
        self.last_prompt_tokens = active_context_input_tokens;
        self
    }
}

/// Header bar widget (1 line height).
pub struct HeaderWidget<'a> {
    data: HeaderData<'a>,
}

impl<'a> HeaderWidget<'a> {
    #[must_use]
    pub fn new(data: HeaderData<'a>) -> Self {
        Self { data }
    }

    fn mode_color(mode: AppMode) -> Color {
        match mode {
            AppMode::Agent => palette::MODE_AGENT,
            AppMode::Yolo => palette::MODE_YOLO,
            AppMode::Plan => palette::MODE_PLAN,
        }
    }

    fn mode_name(mode: AppMode) -> &'static str {
        match mode {
            AppMode::Agent => "Agent",
            AppMode::Yolo => "Yolo",
            AppMode::Plan => "Plan",
        }
    }

    fn mode_fallback(mode: AppMode) -> &'static str {
        match mode {
            AppMode::Agent => "a",
            AppMode::Yolo => "y",
            AppMode::Plan => "p",
        }
    }

    fn core_props(&self) -> HeaderProps<'_> {
        HeaderProps {
            model: self.data.model,
            workspace_name: self.data.workspace_name,
            mode: HeaderMode {
                label: Self::mode_name(self.data.mode),
                fallback: Self::mode_fallback(self.data.mode),
                color: Self::mode_color(self.data.mode),
            },
            is_streaming: self.data.is_streaming,
            context_window: self.data.context_window,
            last_prompt_tokens: self.data.last_prompt_tokens,
            reasoning_effort_label: self.data.reasoning_effort_label,
            provider_label: self.data.provider_label,
            colors: HeaderColors {
                background: self.data.background,
                text_hint: palette::TEXT_HINT,
                text_soft: palette::TEXT_SOFT,
                text_secondary: palette::TEXT_SECONDARY,
                border: palette::BORDER_COLOR,
                accent: palette::DEEPSEEK_SKY,
                status_warning: palette::STATUS_WARNING,
                status_error: palette::STATUS_ERROR,
            },
        }
    }
}

impl Renderable for HeaderWidget<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        CoreHeaderWidget::new(self.core_props()).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        CoreHeaderWidget::new(self.core_props()).desired_height(width)
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderData, HeaderWidget, Renderable};
    use crate::palette;
    use crate::tui::app::AppMode;
    use ratatui::{buffer::Buffer, layout::Rect};

    fn render_header(data: HeaderData<'_>, width: u16) -> String {
        let widget = HeaderWidget::new(data);
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        (0..width).map(|x| buf[(x, 0)].symbol()).collect::<String>()
    }

    #[test]
    fn wide_header_shows_plain_mode_and_single_metadata_cluster() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Agent,
                "deepseek-v4-pro",
                "deepseek-tui",
                false,
                palette::DEEPSEEK_INK,
            ),
            72,
        );

        assert!(rendered.contains("Agent"));
        assert!(rendered.contains("deepseek-tui"));
        assert!(rendered.contains("deepseek-v4-pro"));
        assert!(!rendered.contains("Plan"));
        assert!(!rendered.contains("Yolo"));
    }

    #[test]
    fn streaming_header_integrates_live_state_with_context_signal() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Plan,
                "deepseek-v4-pro",
                "workspace",
                true,
                palette::DEEPSEEK_INK,
            )
            .with_usage(42_000, Some(128_000), 0.0, Some(48_000)),
            72,
        );

        assert!(rendered.contains("Live"));
        assert!(rendered.contains("38%"));
        assert!(rendered.contains("▰"));
    }

    #[test]
    fn narrow_header_keeps_context_percent_visible() {
        let rendered = render_header(
            HeaderData::new(AppMode::Agent, "", "", true, palette::DEEPSEEK_INK).with_usage(
                0,
                Some(128_000),
                0.0,
                Some(48_000),
            ),
            14,
        );

        assert!(rendered.contains('%'));
        assert!(!rendered.contains("Live"));
    }

    #[test]
    fn narrow_header_falls_back_to_mode_without_rendering_all_modes() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Yolo,
                "a-very-long-model-name",
                "workspace",
                false,
                palette::DEEPSEEK_INK,
            ),
            3,
        );

        assert!(rendered.contains('y') || rendered.contains("Yolo"));
        assert!(!rendered.contains("Agent"));
        assert!(!rendered.contains("Plan"));
    }

    #[test]
    fn header_hides_context_signal_when_usage_snapshot_is_missing() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Plan,
                "deepseek-v4-pro",
                "workspace",
                false,
                palette::DEEPSEEK_INK,
            ),
            72,
        );

        assert!(!rendered.contains('%'));
        assert!(!rendered.contains('▰'));
        assert!(!rendered.contains('▱'));
    }

    #[test]
    fn header_caps_context_signal_at_hundred_percent() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Agent,
                "deepseek-v4-pro",
                "workspace",
                false,
                palette::DEEPSEEK_INK,
            )
            .with_usage(0, Some(128_000), 0.0, Some(256_000)),
            72,
        );

        assert!(rendered.contains("100%"));
        assert!(rendered.contains("▰▰▰▰"));
    }

    #[test]
    fn header_shows_provider_chip_when_set() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Agent,
                "deepseek-v4-pro",
                "workspace",
                false,
                palette::DEEPSEEK_INK,
            )
            .with_provider(Some("NIM"))
            .with_reasoning_effort(Some("high")),
            72,
        );

        assert!(rendered.contains("NIM"));
        assert!(rendered.contains("high"));
    }

    #[test]
    fn header_hides_provider_chip_when_default_deepseek() {
        let rendered = render_header(
            HeaderData::new(
                AppMode::Agent,
                "deepseek-v4-pro",
                "workspace",
                false,
                palette::DEEPSEEK_INK,
            )
            .with_provider(None)
            .with_reasoning_effort(Some("high")),
            72,
        );

        assert!(!rendered.contains("NIM"));
        assert!(rendered.contains("high"));
    }
}
