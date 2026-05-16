use crate::palette;
use crate::tui::app::{App, AppMode, ComposerDensity, VimMode};
use crate::{commands, config::COMMON_DEEPSEEK_MODELS};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::Renderable;
use super::menu::{centered_menu_top, fit_menu_cell};

pub(crate) const COMPOSER_PANEL_HEIGHT: u16 = 2;

pub struct ComposerWidget<'a> {
    app: &'a App,
    max_height: u16,
    slash_menu_entries: &'a [SlashMenuEntry],
    mention_menu_entries: &'a [String],
}

impl<'a> ComposerWidget<'a> {
    pub fn new(
        app: &'a App,
        max_height: u16,
        slash_menu_entries: &'a [SlashMenuEntry],
        mention_menu_entries: &'a [String],
    ) -> Self {
        Self {
            app,
            max_height,
            slash_menu_entries,
            mention_menu_entries,
        }
    }

    /// Number of popup rows below the input. Mention and slash menus are
    /// mutually exclusive: the cursor can only sit inside an `@token` OR
    /// a `/cmd` token, not both at once. Mention takes precedence because
    /// the partial-mention check is positional and stricter than slash's
    /// "starts-with-/" check.
    fn active_menu_row_count(&self) -> usize {
        if self.app.is_history_search_active() {
            self.app.history_search_matches().len().max(1)
        } else if !self.mention_menu_entries.is_empty() {
            self.mention_menu_entries.len()
        } else {
            self.slash_menu_entries.len()
        }
    }

    /// Row reservation passed to `composer_height`. When the slash- or
    /// mention-menu is active we lock the composer to its worst-case
    /// envelope so the chat area above doesn't repaint every keystroke
    /// as the matched-entry count shrinks. Pure cosmetic: the menu
    /// itself still renders its actual entries.
    fn active_menu_reserved_rows(&self) -> usize {
        let actual = self.active_menu_row_count();
        if actual == 0 {
            return 0;
        }
        if self.app.is_history_search_active() {
            return actual;
        }
        actual.max(usize::from(self.max_height_cap()))
    }

    fn has_panel(&self, area: Rect) -> bool {
        self.app.composer_border && area.height >= 3 && area.width >= 12
    }

    fn inner_area(&self, area: Rect) -> Rect {
        if self.has_panel(area) {
            Block::default().borders(Borders::ALL).inner(area)
        } else {
            area
        }
    }

    fn mode_color(&self) -> Color {
        match self.app.mode {
            AppMode::Agent => palette::MODE_AGENT,
            AppMode::Yolo => palette::MODE_YOLO,
            AppMode::Plan => palette::MODE_PLAN,
        }
    }

    fn is_game_player_presentation(&self) -> bool {
        self.app
            .game_session
            .as_ref()
            .is_some_and(|session| !session.developer_mode())
    }

    fn title_text(&self, is_draft_mode: bool) -> String {
        if self.app.is_history_search_active() {
            self.app
                .tr(crate::localization::MessageId::HistorySearchTitle)
                .to_string()
        } else if is_draft_mode {
            "Draft".to_string()
        } else if self.is_game_player_presentation() {
            match self.app.active_skill_name.as_deref() {
                Some(name) => format!("Action /skill {name}"),
                None => "Action".to_string(),
            }
        } else {
            "Composer".to_string()
        }
    }

    fn placeholder_text(&self) -> String {
        if self.app.is_history_search_active() {
            self.app
                .tr(crate::localization::MessageId::HistorySearchPlaceholder)
                .to_string()
        } else if self.is_game_player_presentation() {
            match self.app.active_skill_name.as_deref() {
                Some(name) => {
                    format!("Type the content for /skill {name}; Enter sends this action.")
                }
                None => {
                    "Type a player action, dialogue, choice number, or /game rules.".to_string()
                }
            }
        } else {
            self.app
                .tr(crate::localization::MessageId::ComposerPlaceholder)
                .to_string()
        }
    }

    fn max_height_cap(&self) -> u16 {
        composer_max_height(self.app.composer_density)
    }
}

impl Renderable for ComposerWidget<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let background = Style::default().bg(self.app.ui_theme.composer_bg);
        let has_panel = self.has_panel(area);
        let inner_area = self.inner_area(area);
        let input_text = self.app.composer_display_input();
        let input_cursor = self.app.composer_display_cursor();
        let history_search_matches = if self.app.is_history_search_active() {
            self.app.history_search_matches()
        } else {
            Vec::new()
        };
        let menu_lines = self.active_menu_row_count();
        let menu_lines_for_budget = self.active_menu_reserved_rows().max(menu_lines);
        let input_rows_budget =
            composer_input_rows_budget(inner_area.height, menu_lines_for_budget);
        let content_width = usize::from(inner_area.width.max(1));
        let (visible_lines, _cursor_row, _cursor_col) =
            layout_input(input_text, input_cursor, content_width, input_rows_budget);
        let is_draft_mode = input_text.contains('\n') || visible_lines.len() > 1;
        if has_panel {
            let border_color = if input_text.trim().is_empty() {
                palette::BORDER_COLOR
            } else {
                self.mode_color()
            };
            let hint_line = if self.app.is_history_search_active() {
                Some(Line::from(vec![
                    Span::styled(
                        format!(
                            " {}  ",
                            self.app.tr(crate::localization::MessageId::HistoryHintMove)
                        ),
                        Style::default().fg(palette::TEXT_MUTED),
                    ),
                    Span::styled(
                        format!(
                            "{}  ",
                            self.app
                                .tr(crate::localization::MessageId::HistoryHintAccept)
                        ),
                        Style::default().fg(palette::TEXT_MUTED),
                    ),
                    Span::styled(
                        self.app
                            .tr(crate::localization::MessageId::HistoryHintRestore),
                        Style::default().fg(palette::TEXT_MUTED),
                    ),
                ]))
            } else if !self.slash_menu_entries.is_empty() {
                Some(Line::from(vec![
                    Span::styled(" Up/Down move  ", Style::default().fg(palette::TEXT_MUTED)),
                    Span::styled("Tab accept  ", Style::default().fg(palette::TEXT_MUTED)),
                    Span::styled("Esc close", Style::default().fg(palette::TEXT_MUTED)),
                ]))
            } else if !input_text.trim().is_empty() {
                use crate::tui::app::SubmitDisposition;
                let queue_count = self.app.queued_message_count();
                let (label, color) = match self.app.decide_submit_disposition() {
                    SubmitDisposition::Immediate => {
                        if queue_count > 0 {
                            (
                                Some(format!("↵ send ({} queued)", queue_count)),
                                palette::DEEPSEEK_SKY,
                            )
                        } else {
                            (None, palette::TEXT_MUTED)
                        }
                    }
                    SubmitDisposition::Queue => {
                        if self.app.offline_mode {
                            (Some("↵ offline queue".to_string()), palette::STATUS_WARNING)
                        } else {
                            let label = if queue_count > 0 {
                                format!("↵ queue ({} waiting)", queue_count.saturating_add(1))
                            } else {
                                "↵ queue for next turn".to_string()
                            };
                            (Some(label), palette::TEXT_MUTED)
                        }
                    }
                    SubmitDisposition::Steer => (
                        Some("↵ steering (Ctrl+Enter)".to_string()),
                        palette::DEEPSEEK_SKY,
                    ),
                    SubmitDisposition::QueueFollowUp => (
                        Some("↵ queued (Ctrl+Enter to steer)".to_string()),
                        palette::TEXT_MUTED,
                    ),
                };
                label.map(|text| {
                    Line::from(vec![Span::styled(
                        format!(" {text} "),
                        Style::default().fg(color),
                    )])
                })
            } else {
                None
            };

            let mut block = Block::default()
                .title(Line::from(Span::styled(
                    self.title_text(is_draft_mode),
                    Style::default().fg(palette::TEXT_MUTED),
                )))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(background);
            if self.app.composer.vim_enabled {
                let color = match self.app.composer.vim_mode {
                    VimMode::Normal => palette::TEXT_MUTED,
                    VimMode::Insert => palette::DEEPSEEK_SKY,
                    VimMode::Visual => palette::MODE_PLAN,
                };
                let label = self.app.composer.vim_mode.label();
                block = block.title_top(
                    Line::from(Span::styled(label, Style::default().fg(color).bold()))
                        .right_aligned(),
                );
            }
            if let Some(hint_line) = hint_line {
                block = block.title_bottom(hint_line);
            }
            block.render(area, buf);
        } else {
            Block::default().style(background).render(area, buf);
        }

        let mut input_lines = Vec::new();
        if input_text.is_empty() {
            let placeholder = self.placeholder_text();
            input_lines.push(Line::from(Span::styled(
                placeholder.clone(),
                Style::default().fg(palette::TEXT_MUTED).italic(),
            )));
        } else {
            for line in &visible_lines {
                input_lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(palette::TEXT_PRIMARY),
                )));
            }
        }

        let visual_rows = if input_text.is_empty() {
            let placeholder = self.placeholder_text();
            placeholder_visual_lines_for(&placeholder, content_width)
        } else {
            input_lines.len()
        };
        let top_padding = composer_top_padding(visual_rows, input_rows_budget);
        let mut lines = Vec::new();
        for _ in 0..top_padding {
            lines.push(Line::from(""));
        }
        lines.extend(input_lines);

        if self.app.is_history_search_active() {
            if history_search_matches.is_empty() {
                lines.push(Line::from(Span::styled(
                    self.app
                        .tr(crate::localization::MessageId::HistoryNoMatches),
                    Style::default().fg(palette::TEXT_MUTED),
                )));
            } else {
                let selected = self
                    .app
                    .history_search_selected_index()
                    .min(history_search_matches.len().saturating_sub(1));
                let menu_visible_rows = inner_area
                    .height
                    .saturating_sub(visual_rows as u16)
                    .saturating_sub(top_padding as u16)
                    .saturating_sub(1)
                    .max(1) as usize;
                let menu_total = history_search_matches.len();
                let menu_top = centered_menu_top(selected, menu_total, menu_visible_rows);
                let menu_bottom = (menu_top + menu_visible_rows).min(menu_total);

                for (idx, entry) in history_search_matches
                    .iter()
                    .enumerate()
                    .take(menu_bottom)
                    .skip(menu_top)
                {
                    let is_selected = idx == selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(palette::SELECTION_TEXT)
                            .bg(palette::SELECTION_BG)
                    } else {
                        Style::default().fg(palette::TEXT_MUTED)
                    };
                    let marker = if is_selected { "▸" } else { " " };
                    lines.push(Line::from(vec![
                        Span::styled(" ", Style::default()),
                        Span::styled(marker, style),
                        Span::styled(" ", style),
                        Span::styled(entry.clone(), style),
                    ]));
                }
            }
        } else if !self.mention_menu_entries.is_empty() {
            let selected = self
                .app
                .mention_menu_selected
                .min(self.mention_menu_entries.len().saturating_sub(1));
            let menu_visible_rows = inner_area
                .height
                .saturating_sub(visual_rows as u16)
                .saturating_sub(top_padding as u16)
                .saturating_sub(1)
                .max(1) as usize;
            let menu_total = self.mention_menu_entries.len();
            let menu_top = centered_menu_top(selected, menu_total, menu_visible_rows);
            let menu_bottom = (menu_top + menu_visible_rows).min(menu_total);

            for (idx, entry) in self
                .mention_menu_entries
                .iter()
                .enumerate()
                .take(menu_bottom)
                .skip(menu_top)
            {
                let is_selected = idx == selected;
                let style = if is_selected {
                    Style::default()
                        .fg(palette::SELECTION_TEXT)
                        .bg(palette::SELECTION_BG)
                } else {
                    Style::default().fg(palette::TEXT_MUTED)
                };
                let marker = if is_selected { "▸" } else { " " };
                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(marker, style),
                    Span::styled(" ", style),
                    Span::styled(format!("@{entry}"), style),
                ]));
            }
        } else if !self.slash_menu_entries.is_empty() {
            let selected = self
                .app
                .slash_menu_selected
                .min(self.slash_menu_entries.len().saturating_sub(1));
            let menu_visible_rows = inner_area
                .height
                .saturating_sub(visual_rows as u16)
                .saturating_sub(top_padding as u16)
                .saturating_sub(1)
                .max(1) as usize;
            let menu_total = self.slash_menu_entries.len();
            let menu_top = centered_menu_top(selected, menu_total, menu_visible_rows);
            let menu_bottom = (menu_top + menu_visible_rows).min(menu_total);

            let label_width = 22.min(content_width.saturating_sub(4));
            for (idx, entry) in self
                .slash_menu_entries
                .iter()
                .enumerate()
                .take(menu_bottom)
                .skip(menu_top)
            {
                let is_selected = idx == selected;
                let sel_style = if is_selected {
                    Style::default()
                        .fg(palette::SELECTION_TEXT)
                        .bg(palette::SELECTION_BG)
                } else {
                    Style::default().fg(palette::TEXT_MUTED)
                };
                let marker = if is_selected { "▸" } else { " " };
                let name_style = if entry.is_skill && !is_selected {
                    Style::default().fg(palette::DEEPSEEK_SKY)
                } else {
                    sel_style
                };
                let desc_style = if is_selected {
                    Style::default()
                        .fg(palette::SELECTION_TEXT)
                        .bg(palette::SELECTION_BG)
                } else {
                    Style::default().fg(palette::TEXT_DIM)
                };

                let name_display = fit_menu_cell(&entry.name, label_width, true);
                let skill_prefix = if entry.is_skill { "✦" } else { " " };
                let prefix_display_width = 1 + 1 + skill_prefix.width() + label_width + 2;
                let desc_capacity = content_width.saturating_sub(prefix_display_width);
                let desc_display = fit_menu_cell(&entry.description, desc_capacity, false);

                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(marker, sel_style),
                    Span::styled(skill_prefix, name_style),
                    Span::styled(name_display, name_style),
                    Span::styled("  ", desc_style),
                    Span::styled(desc_display, desc_style),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .style(background)
            .wrap(Wrap { trim: false });
        paragraph.render(inner_area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        composer_height(
            self.app.composer_display_input(),
            width,
            self.max_height.min(self.max_height_cap()),
            self.active_menu_reserved_rows(),
            self.app.composer_density,
            self.app.composer_border,
        )
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        let inner_area = self.inner_area(area);
        let input_text = self.app.composer_display_input();
        let input_cursor = self.app.composer_display_cursor();
        let content_width = usize::from(inner_area.width.max(1));
        let input_rows_budget =
            composer_input_rows_budget(inner_area.height, self.active_menu_reserved_rows());

        let (visible_lines, cursor_row, cursor_col) =
            layout_input(input_text, input_cursor, content_width, input_rows_budget);
        let visual_rows = if input_text.is_empty() {
            let placeholder = self.placeholder_text();
            placeholder_visual_lines_for(&placeholder, content_width)
        } else {
            visible_lines.len()
        };
        let top_padding = composer_top_padding(visual_rows, input_rows_budget);

        let cursor_x = area
            .x
            .saturating_add(inner_area.x.saturating_sub(area.x))
            .saturating_add(u16::try_from(cursor_col).unwrap_or(u16::MAX));
        let cursor_y = area
            .y
            .saturating_add(inner_area.y.saturating_sub(area.y))
            .saturating_add(u16::try_from(top_padding + cursor_row).unwrap_or(u16::MAX));
        if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
            Some((cursor_x, cursor_y))
        } else {
            None
        }
    }
}

fn composer_input_rows_budget(inner_height: u16, extra_lines: usize) -> usize {
    usize::from(inner_height).saturating_sub(extra_lines).max(1)
}

pub(crate) fn composer_top_padding(content_lines: usize, rows_budget: usize) -> usize {
    rows_budget.saturating_sub(content_lines.clamp(1, rows_budget))
}

#[cfg(test)]
const COMPOSER_PLACEHOLDER: &str = "Write a task or use /.";

#[cfg(test)]
pub(crate) fn placeholder_visual_lines(content_width: usize) -> usize {
    placeholder_visual_lines_for(COMPOSER_PLACEHOLDER, content_width)
}

fn placeholder_visual_lines_for(placeholder: &str, content_width: usize) -> usize {
    wrap_text(placeholder, content_width).len().max(1)
}

pub(crate) fn composer_min_input_rows(density: ComposerDensity) -> usize {
    match density {
        ComposerDensity::Compact => 2,
        ComposerDensity::Comfortable => 3,
        ComposerDensity::Spacious => 4,
    }
}

pub(crate) fn composer_max_height(density: ComposerDensity) -> u16 {
    match density {
        ComposerDensity::Compact => 7,
        ComposerDensity::Comfortable => 9,
        ComposerDensity::Spacious => 12,
    }
}

pub(crate) fn composer_height(
    input: &str,
    width: u16,
    available_height: u16,
    extra_lines: usize,
    density: ComposerDensity,
    show_panel: bool,
) -> u16 {
    let has_panel = show_panel && available_height >= 3 && width >= 12;
    let chrome_height = if has_panel {
        usize::from(COMPOSER_PANEL_HEIGHT)
    } else {
        0
    };
    let content_width = if has_panel {
        usize::from(width.saturating_sub(2).max(1))
    } else {
        usize::from(width.max(1))
    };
    let mut line_count = wrap_input_lines(input, content_width).len();
    if line_count == 0 {
        line_count = 1;
    }
    if has_panel {
        line_count = line_count.max(composer_min_input_rows(density));
    }
    line_count = line_count
        .saturating_add(extra_lines)
        .saturating_add(chrome_height);
    let max_height = usize::from(available_height.clamp(1, composer_max_height(density)));
    line_count.clamp(1, max_height).try_into().unwrap_or(1)
}

/// A single entry in the slash-command autocomplete popup.
pub(crate) struct SlashMenuEntry {
    pub name: String,
    pub description: String,
    pub is_skill: bool,
}

const GAME_ACTION_SKILL_DESCRIPTION_PREFIX: &str = "Game action:";

pub(crate) fn slash_completion_hints(
    input: &str,
    limit: usize,
    cached_skills: &[(String, String)],
    locale: crate::localization::Locale,
) -> Vec<SlashMenuEntry> {
    if !input.starts_with('/') {
        return Vec::new();
    }

    let prefix = input.trim_start_matches('/');
    let completing_skill_arg = prefix.strip_prefix("skill ").map(str::trim_start);
    if input.contains(char::is_whitespace) && completing_skill_arg.is_none() {
        return Vec::new();
    }
    let mut entries: Vec<SlashMenuEntry> = Vec::new();

    if completing_skill_arg.is_none() {
        for name in commands::all_command_names_matching(prefix) {
            let command_key = name.trim_start_matches('/');
            let description = if let Some(info) = commands::get_command_info(command_key) {
                info.description_for(locale).to_string()
            } else {
                String::from("User-defined command")
            };
            entries.push(SlashMenuEntry {
                name,
                description,
                is_skill: false,
            });
        }
    }

    let skill_prefix = completing_skill_arg.unwrap_or(prefix);
    let prefix_lower = skill_prefix.to_ascii_lowercase();
    for (skill_name, skill_desc) in cached_skills {
        let skill_name_lower = skill_name.to_ascii_lowercase();
        let command_prefix_matches = completing_skill_arg.is_none()
            && (prefix_lower.is_empty()
                || "skill".starts_with(&prefix_lower)
                || skill_name_lower.starts_with(&prefix_lower));
        let skill_arg_matches =
            completing_skill_arg.is_some() && skill_name_lower.starts_with(&prefix_lower);
        if command_prefix_matches || skill_arg_matches {
            entries.push(SlashMenuEntry {
                name: format!("/skill {skill_name}"),
                description: skill_desc.clone(),
                is_skill: true,
            });
        }
    }

    if entries.iter().any(|e| e.name == "/model") && prefix_lower.eq_ignore_ascii_case("model") {
        for model_name in COMMON_DEEPSEEK_MODELS {
            entries.push(SlashMenuEntry {
                name: format!("/model {model_name}"),
                description: String::from("Switch to this model"),
                is_skill: false,
            });
        }
    }

    let prioritize_game_actions = prefix.starts_with("skill");
    entries.sort_by(|a, b| {
        let rank = |entry: &SlashMenuEntry| -> u8 {
            if prioritize_game_actions
                && entry
                    .description
                    .starts_with(GAME_ACTION_SKILL_DESCRIPTION_PREFIX)
            {
                0
            } else if entry.is_skill {
                2
            } else {
                1
            }
        };
        rank(a).cmp(&rank(b)).then_with(|| a.name.cmp(&b.name))
    });
    entries.dedup_by(|a, b| a.name == b.name);
    entries.into_iter().take(limit).collect()
}

pub(crate) fn layout_input(
    input: &str,
    cursor: usize,
    width: usize,
    max_height: usize,
) -> (Vec<String>, usize, usize) {
    let mut lines = wrap_input_lines(input, width);
    if lines.is_empty() {
        lines.push(String::new());
    }
    let (cursor_row, cursor_col) = cursor_row_col(input, cursor, width.max(1));

    let max_height = max_height.max(1);
    let mut start = 0usize;
    if cursor_row >= max_height {
        start = cursor_row + 1 - max_height;
    }
    if start + max_height > lines.len() {
        start = lines.len().saturating_sub(max_height);
    }
    let visible = lines
        .into_iter()
        .skip(start)
        .take(max_height)
        .collect::<Vec<_>>();
    let visible_cursor_row = cursor_row.saturating_sub(start);

    (
        visible,
        visible_cursor_row,
        cursor_col.min(width.saturating_sub(1)),
    )
}

pub(crate) fn cursor_row_col(input: &str, cursor: usize, width: usize) -> (usize, usize) {
    let mut row = 0usize;
    let mut col = 0usize;
    let mut char_idx = 0usize;

    for grapheme in input.graphemes(true) {
        if char_idx >= cursor {
            break;
        }
        let grapheme_chars = grapheme.chars().count();
        let next_char_idx = char_idx.saturating_add(grapheme_chars);
        let cursor_inside = cursor < next_char_idx;

        if grapheme == "\n" {
            row += 1;
            col = 0;
            char_idx = next_char_idx;
            if cursor_inside {
                break;
            }
            continue;
        }

        let grapheme_width = grapheme.width();
        if col + grapheme_width > width && col != 0 {
            row += 1;
            col = 0;
        }
        col += grapheme_width;
        if col >= width {
            row += 1;
            col = 0;
        }
        if cursor_inside {
            break;
        }
        char_idx = next_char_idx;
    }

    (row, col)
}

pub(crate) fn wrap_input_lines(input: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if input.is_empty() {
        return lines;
    }

    for raw in input.split('\n') {
        let wrapped = wrap_text(raw, width);
        if wrapped.is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(wrapped);
        }
    }

    lines
}

pub(crate) fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for grapheme in text.graphemes(true) {
        if grapheme == "\n" {
            lines.push(current);
            current = String::new();
            current_width = 0;
            continue;
        }

        let grapheme_width = grapheme.width();
        if current_width + grapheme_width > width && current_width != 0 {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }

        current.push_str(grapheme);
        current_width += grapheme_width;

        if current_width >= width {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }
    }

    lines.push(current);
    lines
}
