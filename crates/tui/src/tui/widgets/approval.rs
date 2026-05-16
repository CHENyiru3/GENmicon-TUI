use crate::localization::Locale;
use crate::palette;
use crate::tui::approval::{
    ApprovalRequest, ApprovalView, ElevationOption, ElevationRequest, RiskLevel, ToolCategory,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

use super::Renderable;

/// Codex-style full-screen approval takeover (#129).
///
/// The widget reads its mutable state (selected option, staged
/// confirmation) directly from the [`ApprovalView`] so the destructive
/// variant can render its "Press Y again to confirm" banner without
/// touching internal fields. Rendering reflows to fill most of the
/// transcript area instead of a centered popup; on small terminals it
/// falls back to a 65x22 card so existing snapshot tests still see a
/// coherent layout.
pub struct ApprovalWidget<'a> {
    request: &'a ApprovalRequest,
    view: &'a ApprovalView,
}

impl<'a> ApprovalWidget<'a> {
    pub fn new(request: &'a ApprovalRequest, view: &'a ApprovalView) -> Self {
        Self { request, view }
    }
}

/// Layout pad around the takeover card. Generous so the modal feels
/// like a takeover rather than a popup, but never larger than the
/// terminal can hold.
const APPROVAL_CARD_HORIZONTAL_PAD: u16 = 6;
const APPROVAL_CARD_VERTICAL_PAD: u16 = 2;
/// Minimum card height: anything tighter and the destructive variant's
/// confirmation banner overlaps the option list.
const APPROVAL_CARD_MIN_HEIGHT: u16 = 18;
/// Minimum card width: anything tighter makes approval copy wrap too
/// aggressively on small terminals.
const APPROVAL_CARD_MIN_WIDTH: u16 = 40;
/// Maximum card height: taller cards stop reading like a focused
/// takeover and waste vertical space on large terminals.
const APPROVAL_CARD_MAX_HEIGHT: u16 = 28;
/// Maximum card width: readability craters past this on wide terminals.
const APPROVAL_CARD_MAX_WIDTH: u16 = 96;

impl Renderable for ApprovalWidget<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let card_area = compute_takeover_area(area);
        Clear.render(card_area, buf);

        let risk = self.request.risk;
        let locale = self.view.locale();
        let palette_colors = approval_palette(risk);
        let mut lines: Vec<Line<'static>> = Vec::with_capacity(20);

        // Header: stakes badge + tool identifier. The badge is the
        // first thing the eye lands on.
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!(" {} ", risk_badge_text(risk, locale)),
                Style::default()
                    .fg(palette::DEEPSEEK_INK)
                    .bg(palette_colors.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                self.request.tool_name.clone(),
                Style::default()
                    .fg(palette::DEEPSEEK_SKY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Category line: English remains the baseline while localized
        // sessions get the same risk category in their UI language.
        let (cat_label, cat_color) = category_label_for(self.request.category, locale);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(label_type(locale), Style::default().fg(palette::TEXT_HINT)),
            Span::styled(
                cat_label,
                Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(label_about(locale), Style::default().fg(palette::TEXT_HINT)),
            Span::styled(
                self.request.description_for_locale(locale),
                Style::default().fg(palette::TEXT_BODY),
            ),
        ]));
        for impact in self.request.impacts_for_locale(locale).into_iter().take(4) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    label_impact(locale),
                    Style::default().fg(palette::TEXT_HINT),
                ),
                Span::styled(impact, Style::default().fg(palette::TEXT_BODY)),
            ]));
        }

        lines.push(Line::from(""));
        let params_str = self.request.params_display();
        let params_width = card_area.width.saturating_sub(14) as usize;
        let params_truncated =
            crate::utils::truncate_with_ellipsis(&params_str, params_width.max(20), "...");
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                label_params(locale),
                Style::default().fg(palette::TEXT_HINT),
            ),
            Span::styled(
                params_truncated,
                Style::default().fg(palette::TEXT_SECONDARY),
            ),
        ]));

        lines.push(Line::from(""));

        let options = approval_options_for(risk, locale);
        let pending = self.view.pending_confirm();

        for (i, opt) in options.iter().enumerate() {
            let is_selected = i == self.view.selected();
            let staged = pending.is_some_and(|p| p == opt.option);
            let label_color = if opt.dangerous {
                palette_colors.accent
            } else {
                palette::TEXT_BODY
            };

            let row_style = if is_selected {
                Style::default()
                    .fg(palette::SELECTION_TEXT)
                    .bg(palette::SELECTION_BG)
            } else {
                Style::default()
            };

            let mut spans = vec![
                Span::raw("  "),
                Span::styled(
                    format!("[{}] ", opt.key_hint),
                    Style::default()
                        .fg(palette_colors.shortcut)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(opt.label.to_string(), row_style.fg(label_color)),
            ];
            if staged {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    staged_marker(locale),
                    Style::default()
                        .fg(palette_colors.accent)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(spans));
        }

        lines.push(Line::from(""));
        match (risk, pending) {
            (RiskLevel::Benign, _) => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        single_key_prefix(locale),
                        Style::default().fg(palette::TEXT_HINT),
                    ),
                    Span::styled(
                        single_key_value(locale),
                        Style::default()
                            .fg(palette_colors.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        footer_controls(locale),
                        Style::default().fg(palette::TEXT_HINT),
                    ),
                ]));
            }
            (RiskLevel::Destructive, Some(opt)) => {
                let again_key = match opt {
                    crate::tui::approval::ApprovalOption::ApproveOnce => confirm_key_once(locale),
                    crate::tui::approval::ApprovalOption::ApproveAlways => {
                        confirm_key_always(locale)
                    }
                    _ => "Enter",
                };
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        destructive_confirm_prefix(locale),
                        Style::default()
                            .fg(palette_colors.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        again_key.to_string(),
                        Style::default()
                            .fg(palette::DEEPSEEK_INK)
                            .bg(palette_colors.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        destructive_confirm_suffix(locale),
                        Style::default().fg(palette::TEXT_HINT),
                    ),
                ]));
            }
            (RiskLevel::Destructive, None) => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        two_key_prefix(locale),
                        Style::default().fg(palette::TEXT_HINT),
                    ),
                    Span::styled(
                        two_key_value(locale),
                        Style::default()
                            .fg(palette_colors.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        footer_controls(locale),
                        Style::default().fg(palette::TEXT_HINT),
                    ),
                ]));
            }
        }

        let title = format!(
            " {} {} — {} ",
            risk_badge_text(risk, locale),
            approval_word(locale),
            self.request.tool_name
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette_colors.border))
            .style(Style::default().bg(palette::DEEPSEEK_INK))
            .padding(Padding::uniform(1));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        paragraph.render(card_area, buf);

        if matches!(risk, RiskLevel::Destructive) {
            paint_left_rail(card_area, buf, palette_colors.accent);
        }
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}

/// Compute the card rect inside `area`. Always centered; pad on every
/// side so the takeover reads as a takeover but a small terminal still
/// stays inside the buffer. Very small terminals may truncate the card
/// content, but rendering must never address cells outside `area`.
pub(crate) fn compute_takeover_area(area: Rect) -> Rect {
    let avail_width = area.width.saturating_sub(APPROVAL_CARD_HORIZONTAL_PAD * 2);
    let avail_height = area.height.saturating_sub(APPROVAL_CARD_VERTICAL_PAD * 2);
    let card_width = APPROVAL_CARD_MAX_WIDTH
        .min(avail_width)
        .max(APPROVAL_CARD_MIN_WIDTH)
        .min(area.width);
    let card_height = APPROVAL_CARD_MIN_HEIGHT
        .max(avail_height.min(APPROVAL_CARD_MAX_HEIGHT))
        .min(area.height);
    let x = area.x + (area.width.saturating_sub(card_width)) / 2;
    let y = area.y + (area.height.saturating_sub(card_height)) / 2;
    Rect {
        x,
        y,
        width: card_width,
        height: card_height,
    }
}

/// Paint a single-column accent on the inside-left of the card. Only
/// touches cells that already exist in the buffer area.
fn paint_left_rail(card: Rect, buf: &mut Buffer, color: Color) {
    if card.width < 2 || card.height < 4 {
        return;
    }
    let rail_x = card.x + 1;
    let top = card.y + 1;
    let bot = card.y + card.height.saturating_sub(2);
    for y in top..=bot {
        if y >= buf.area.y + buf.area.height {
            break;
        }
        let cell = &mut buf[(rail_x, y)];
        cell.set_char('\u{2503}');
        cell.set_style(Style::default().fg(color).bg(palette::DEEPSEEK_INK));
    }
}

struct ApprovalColors {
    border: Color,
    accent: Color,
    shortcut: Color,
}

fn approval_palette(risk: RiskLevel) -> ApprovalColors {
    match risk {
        RiskLevel::Benign => ApprovalColors {
            border: palette::BORDER_COLOR,
            accent: palette::DEEPSEEK_SKY,
            shortcut: palette::DEEPSEEK_SKY,
        },
        RiskLevel::Destructive => ApprovalColors {
            border: palette::DEEPSEEK_RED,
            accent: palette::DEEPSEEK_RED,
            shortcut: palette::STATUS_WARNING,
        },
    }
}

fn risk_badge_text(risk: RiskLevel, locale: Locale) -> &'static str {
    match (locale, risk) {
        (Locale::ZhHans, RiskLevel::Benign) => "审查",
        (Locale::ZhHans, RiskLevel::Destructive) => "破坏性",
        (_, RiskLevel::Benign) => "REVIEW",
        (_, RiskLevel::Destructive) => "DESTRUCTIVE",
    }
}

fn category_label_for(category: ToolCategory, locale: Locale) -> (&'static str, Color) {
    match (locale, category) {
        (Locale::ZhHans, ToolCategory::Safe) => ("安全", palette::STATUS_SUCCESS),
        (Locale::ZhHans, ToolCategory::FileWrite) => ("文件写入", palette::STATUS_WARNING),
        (Locale::ZhHans, ToolCategory::Shell) => ("Shell 命令", palette::STATUS_ERROR),
        (Locale::ZhHans, ToolCategory::Network) => ("网络", palette::STATUS_WARNING),
        (Locale::ZhHans, ToolCategory::McpRead) => ("MCP 读取", palette::DEEPSEEK_SKY),
        (Locale::ZhHans, ToolCategory::McpAction) => ("MCP 操作", palette::STATUS_WARNING),
        (Locale::ZhHans, ToolCategory::Unknown) => ("未知", palette::STATUS_ERROR),
        (_, ToolCategory::Safe) => ("Safe", palette::STATUS_SUCCESS),
        (_, ToolCategory::FileWrite) => ("File Write", palette::STATUS_WARNING),
        (_, ToolCategory::Shell) => ("Shell Command", palette::STATUS_ERROR),
        (_, ToolCategory::Network) => ("Network", palette::STATUS_WARNING),
        (_, ToolCategory::McpRead) => ("MCP Read", palette::DEEPSEEK_SKY),
        (_, ToolCategory::McpAction) => ("MCP Action", palette::STATUS_WARNING),
        (_, ToolCategory::Unknown) => ("Unknown", palette::STATUS_ERROR),
    }
}

fn approval_word(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "审批",
        _ => "approval",
    }
}

fn label_type(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "类型：",
        _ => "Type: ",
    }
}

fn label_about(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "说明：",
        _ => "About:  ",
    }
}

fn label_impact(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "影响：",
        _ => "Impact: ",
    }
}

fn label_params(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "参数：",
        _ => "Params: ",
    }
}

fn staged_marker(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "(待确认)",
        _ => "(staged)",
    }
}

fn single_key_prefix(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "单键批准：",
        _ => "Single key approves: ",
    }
}

fn single_key_value(_locale: Locale) -> &'static str {
    "Enter / 1 / y"
}

fn footer_controls(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "  ·  v：完整参数  ·  Esc：中止",
        _ => "  ·  v: full params  ·  Esc: abort",
    }
}

fn destructive_confirm_prefix(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "确认破坏性操作：再次按 ",
        _ => "Confirm destructive action — press ",
    }
}

fn destructive_confirm_suffix(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => " 执行；按其他键取消。",
        _ => " again to commit, anything else cancels.",
    }
}

fn confirm_key_once(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "Enter 或 y",
        _ => "Enter or y",
    }
}

fn confirm_key_always(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "Enter 或 a",
        _ => "Enter or a",
    }
}

fn two_key_prefix(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "两次按键确认：",
        _ => "Two keys to approve: ",
    }
}

fn two_key_value(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "先按 y/a，再按一次 y/a",
        _ => "y/a then y/a again",
    }
}

struct ApprovalOptionRow {
    option: crate::tui::approval::ApprovalOption,
    label: &'static str,
    key_hint: &'static str,
    dangerous: bool,
}

fn approval_options_for(risk: RiskLevel, locale: Locale) -> [ApprovalOptionRow; 4] {
    use crate::tui::approval::ApprovalOption as O;
    let dangerous = matches!(risk, RiskLevel::Destructive);
    [
        ApprovalOptionRow {
            option: O::ApproveOnce,
            label: option_approve_once(locale),
            key_hint: "1 / y",
            dangerous,
        },
        ApprovalOptionRow {
            option: O::ApproveAlways,
            label: option_approve_always(locale),
            key_hint: "2 / a",
            dangerous,
        },
        ApprovalOptionRow {
            option: O::Deny,
            label: option_deny(locale),
            key_hint: "3 / d / n",
            dangerous: false,
        },
        ApprovalOptionRow {
            option: O::Abort,
            label: option_abort(locale),
            key_hint: "Esc",
            dangerous: false,
        },
    ]
}

fn option_approve_once(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "仅本次批准",
        _ => "Approve once",
    }
}

fn option_approve_always(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "本会话同类自动批准",
        _ => "Approve always for this kind",
    }
}

fn option_deny(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "拒绝本次调用",
        _ => "Deny this call",
    }
}

fn option_abort(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhHans => "中止本轮",
        _ => "Abort the turn",
    }
}

pub struct ElevationWidget<'a> {
    request: &'a ElevationRequest,
    selected: usize,
}

impl<'a> ElevationWidget<'a> {
    pub fn new(request: &'a ElevationRequest, selected: usize) -> Self {
        Self { request, selected }
    }
}

impl Renderable for ElevationWidget<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 70.min(area.width.saturating_sub(4));
        let popup_height = 22.min(area.height.saturating_sub(4));
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        Clear.render(popup_area, buf);

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  ⚠ Sandbox Denied ",
                Style::default()
                    .fg(palette::STATUS_ERROR)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Tool: "),
                Span::styled(
                    &self.request.tool_name,
                    Style::default()
                        .fg(palette::DEEPSEEK_SKY)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        if let Some(ref command) = self.request.command {
            let cmd_display = crate::utils::truncate_with_ellipsis(command, 45, "...");
            lines.push(Line::from(vec![
                Span::raw("  Cmd:  "),
                Span::styled(cmd_display, Style::default().fg(palette::TEXT_MUTED)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  Reason: "),
            Span::styled(
                &self.request.denial_reason,
                Style::default().fg(palette::STATUS_WARNING),
            ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Impact if approved:",
            Style::default().fg(palette::TEXT_MUTED),
        )));
        if self
            .request
            .options
            .iter()
            .any(|option| matches!(option, ElevationOption::WithNetwork))
        {
            lines.push(Line::from(Span::styled(
                "    - network retry enables outbound downloads and HTTP requests",
                Style::default().fg(palette::TEXT_PRIMARY),
            )));
        }
        if self
            .request
            .options
            .iter()
            .any(|option| matches!(option, ElevationOption::WithWriteAccess(_)))
        {
            lines.push(Line::from(Span::styled(
                "    - write retry expands writable filesystem scope for this tool call",
                Style::default().fg(palette::TEXT_PRIMARY),
            )));
        }
        lines.push(Line::from(Span::styled(
            "    - full access removes sandbox restrictions entirely for this retry",
            Style::default().fg(palette::TEXT_PRIMARY),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Choose how to proceed:",
            Style::default().fg(palette::TEXT_MUTED),
        )));
        lines.push(Line::from(""));

        for (i, option) in self.request.options.iter().enumerate() {
            let is_selected = i == self.selected;
            let style = if is_selected {
                Style::default()
                    .fg(palette::SELECTION_TEXT)
                    .bg(palette::SELECTION_BG)
            } else {
                Style::default()
            };

            let key = match option {
                ElevationOption::WithNetwork => "n",
                ElevationOption::WithWriteAccess(_) => "w",
                ElevationOption::FullAccess => "f",
                ElevationOption::Abort => "a",
            };

            let label_color = match option {
                ElevationOption::Abort => palette::TEXT_MUTED,
                ElevationOption::FullAccess => palette::STATUS_ERROR,
                _ => palette::TEXT_PRIMARY,
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("[{key}] "),
                    Style::default().fg(palette::STATUS_SUCCESS),
                ),
                Span::styled(option.label(), style.fg(label_color)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(
                    option.description(),
                    Style::default().fg(palette::TEXT_MUTED),
                ),
            ]));
        }

        let block = Block::default()
            .title(" Sandbox Elevation Required ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette::BORDER_COLOR))
            .style(Style::default().bg(palette::DEEPSEEK_INK))
            .padding(Padding::uniform(1));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        paragraph.render(popup_area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}
