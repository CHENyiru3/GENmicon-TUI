use std::path::{Component, Path, PathBuf};

use deepseek_game::render::{AsciiArtFrame, AsciiArtSource};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::game::{GameLanguage, GameSession, LoadedGameSession};
use crate::palette;
use crate::tui::app::{App, GameConsoleFocus, GameConsoleState};
use crate::tui::history::HistoryCell;

pub struct GameConsoleWidget<'a> {
    session: Option<&'a GameSession>,
    player_log: Vec<String>,
    console: GameConsoleState,
    is_loading: bool,
    background: Color,
}

#[derive(Debug, Clone, Copy)]
struct StyledArtCell {
    symbol: char,
    style: Style,
}

#[derive(Debug, Clone)]
struct RenderableArtFrame {
    ratio_cols: u16,
    ratio_rows: u16,
    lines: Vec<Vec<StyledArtCell>>,
}

impl<'a> GameConsoleWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            session: app.game_session.as_ref(),
            player_log: player_log(app),
            console: app.game_console.clone(),
            is_loading: app.is_loading,
            background: app.ui_theme.surface_bg,
        }
    }

    #[cfg(test)]
    fn from_parts(session: Option<&'a GameSession>, player_log: Vec<String>) -> Self {
        Self {
            session,
            player_log,
            console: GameConsoleState::default(),
            is_loading: false,
            background: palette::DEEPSEEK_INK,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::default().bg(self.background))
            .render(area, buf);

        let Some(session) = self.session else {
            render_text_panel(
                "Game Console",
                &["No game session is active.".to_string()],
                0,
                false,
                area,
                buf,
                self.background,
            );
            return;
        };

        match session {
            GameSession::Loaded(session) => self.render_loaded(session, area, buf),
            GameSession::Notice(notice) => render_text_panel(
                "Game Console",
                std::slice::from_ref(&notice.message),
                0,
                false,
                area,
                buf,
                self.background,
            ),
        }
    }

    fn render_loaded(&self, session: &LoadedGameSession, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 8 {
            let lines = vec![
                session.title.clone(),
                format!("Save {} r{}", session.save_id, session.revision),
                session.view.scene.clone(),
            ];
            render_text_panel("Game", &lines, 0, false, area, buf, self.background);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        render_game_header(session, self.is_loading, chunks[0], buf, self.background);

        if chunks[1].width >= 120 {
            self.render_wide(session, chunks[1], buf);
        } else if chunks[1].width >= 80 {
            self.render_medium(session, chunks[1], buf);
        } else {
            self.render_narrow(session, chunks[1], buf);
        }
    }

    fn render_wide(&self, session: &LoadedGameSession, area: Rect, buf: &mut Buffer) {
        let bottom_height = area.height.saturating_mul(38).saturating_div(100).max(8);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(bottom_height)])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(rows[0]);
        self.render_scene_panel(session, top[0], buf);
        render_figure_panel(session, top[1], buf, self.background);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(rows[1]);
        render_text_panel(
            panel_label(session, "Dialogue", "对话"),
            &dialogue_lines(session, &self.player_log),
            self.console.dialogue_scroll,
            self.console.focus == GameConsoleFocus::Dialogue,
            bottom[0],
            buf,
            self.background,
        );
        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(26),
                Constraint::Percentage(26),
                Constraint::Percentage(14),
            ])
            .split(bottom[1]);
        render_text_panel(
            panel_label(session, "Choices", "选择"),
            &choices_lines(session),
            0,
            false,
            side[0],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Status", "状态"),
            &status_lines(session),
            0,
            false,
            side[1],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Tasks", "任务"),
            &tasks_lines(session),
            0,
            false,
            side[2],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Items", "物品"),
            &items_lines(session),
            0,
            false,
            side[3],
            buf,
            self.background,
        );
    }

    fn render_medium(&self, session: &LoadedGameSession, area: Rect, buf: &mut Buffer) {
        let hero_height = area.height.saturating_mul(46).saturating_div(100).max(10);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(hero_height), Constraint::Min(8)])
            .split(area);
        let hero = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(56), Constraint::Percentage(44)])
            .split(rows[0]);
        self.render_scene_panel(session, hero[0], buf);
        render_figure_panel(session, hero[1], buf, self.background);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[1]);
        render_text_panel(
            panel_label(session, "Dialogue", "对话"),
            &dialogue_lines(session, &self.player_log),
            self.console.dialogue_scroll,
            self.console.focus == GameConsoleFocus::Dialogue,
            bottom[0],
            buf,
            self.background,
        );
        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Percentage(28),
                Constraint::Percentage(30),
            ])
            .split(bottom[1]);
        render_text_panel(
            panel_label(session, "Choices", "选择"),
            &choices_lines(session),
            0,
            false,
            side[0],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Status", "状态"),
            &status_lines(session),
            0,
            false,
            side[1],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Tasks", "任务"),
            &tasks_lines(session),
            0,
            false,
            side[2],
            buf,
            self.background,
        );
    }

    fn render_narrow(&self, session: &LoadedGameSession, area: Rect, buf: &mut Buffer) {
        let scene_height = area.height.saturating_mul(22).saturating_div(100).max(4);
        let figure_height = area.height.saturating_mul(30).saturating_div(100).max(5);
        let info_height = area.height.saturating_mul(18).saturating_div(100).max(4);
        let choice_height = area.height.saturating_mul(16).saturating_div(100).max(3);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(scene_height),
                Constraint::Length(figure_height),
                Constraint::Length(info_height),
                Constraint::Min(5),
                Constraint::Length(choice_height),
            ])
            .split(area);
        self.render_scene_panel(session, rows[0], buf);
        render_figure_panel(session, rows[1], buf, self.background);
        render_text_panel(
            panel_label(session, "Status / Tasks", "状态 / 任务"),
            &compact_info_lines(session),
            0,
            false,
            rows[2],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Dialogue", "对话"),
            &dialogue_lines(session, &self.player_log),
            self.console.dialogue_scroll,
            self.console.focus == GameConsoleFocus::Dialogue,
            rows[3],
            buf,
            self.background,
        );
        render_text_panel(
            panel_label(session, "Choices", "选择"),
            &choices_lines(session),
            0,
            false,
            rows[4],
            buf,
            self.background,
        );
    }

    fn render_scene_panel(&self, session: &LoadedGameSession, area: Rect, buf: &mut Buffer) {
        render_text_panel(
            panel_label(session, "Scene", "场景"),
            &scene_lines(session),
            self.console.progress_scroll,
            self.console.focus == GameConsoleFocus::Progress,
            area,
            buf,
            self.background,
        );
    }
}

fn render_game_header(
    session: &LoadedGameSession,
    is_loading: bool,
    area: Rect,
    buf: &mut Buffer,
    background: Color,
) {
    let state = if session.language.is_chinese() {
        if is_loading { "处理中" } else { "就绪" }
    } else if is_loading {
        "resolving"
    } else {
        "ready"
    };
    let text = if session.language.is_chinese() {
        format!(
            " {} | 存档 {} | 修订 {} | {} ",
            session.title, session.save_id, session.revision, state
        )
    } else {
        format!(
            " {} | save {} | revision {} | {} ",
            session.title, session.save_id, session.revision, state
        )
    };
    Paragraph::new(Line::from(vec![Span::styled(
        fit_line(&text, area.width as usize),
        Style::default()
            .fg(palette::TEXT_PRIMARY)
            .bg(background)
            .add_modifier(Modifier::BOLD),
    )]))
    .style(Style::default().bg(background))
    .render(area, buf);
}

fn panel_label<'a>(session: &LoadedGameSession, en: &'a str, zh: &'a str) -> &'a str {
    if session.language.is_chinese() {
        zh
    } else {
        en
    }
}

fn render_figure_panel(
    session: &LoadedGameSession,
    area: Rect,
    buf: &mut Buffer,
    background: Color,
) {
    let loaded_source = session
        .view
        .figure_art_source
        .as_ref()
        .and_then(|source| load_ascii_art_source(&session.game_root, source));
    let embedded_art = session
        .view
        .figure_art
        .as_ref()
        .map(renderable_from_plain_frame);
    render_art_or_text_panel(
        &session.view.figure_title,
        loaded_source.as_ref().or(embedded_art.as_ref()),
        &session.view.figure,
        area,
        buf,
        background,
    );
}

fn render_art_or_text_panel(
    title: &str,
    art: Option<&RenderableArtFrame>,
    fallback: &str,
    area: Rect,
    buf: &mut Buffer,
    background: Color,
) {
    let block = panel_block(title, background, false);
    let inner = block.inner(area);
    block.render(area, buf);

    if let Some(frame) = art {
        let fitted = fixed_ratio_rect(inner, frame.ratio_cols, frame.ratio_rows);
        if fitted.width > 0 && fitted.height > 0 {
            let scaled = scale_renderable_art_lines(&frame.lines, fitted.width, fitted.height);
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
                    .style(Style::default().bg(background))
                    .render(art_area, buf);
                return;
            }
            let art_area = Rect {
                x: fitted.x,
                y: fitted.y,
                width: fitted.width,
                height: fitted.height,
            };
            render_lines(fallback_lines(fallback), 0, art_area, buf, background);
            return;
        }
    }

    render_lines(fallback_lines(fallback), 0, inner, buf, background);
}

fn render_text_panel(
    title: &str,
    lines: &[String],
    scroll: usize,
    focused: bool,
    area: Rect,
    buf: &mut Buffer,
    background: Color,
) {
    let block = panel_block(title, background, focused);
    let inner = block.inner(area);
    block.render(area, buf);
    render_lines(lines.to_vec(), scroll, inner, buf, background);
}

fn render_lines(
    lines: Vec<String>,
    scroll: usize,
    area: Rect,
    buf: &mut Buffer,
    background: Color,
) {
    let total_visual_lines = if lines.is_empty() {
        1
    } else {
        lines
            .iter()
            .map(|line| wrapped_line_count(line, area.width))
            .sum()
    };
    let content = if lines.is_empty() {
        vec![Line::from(Span::styled(
            "No visible information yet.",
            Style::default().fg(palette::TEXT_MUTED),
        ))]
    } else {
        lines
            .into_iter()
            .map(|line| {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(palette::TEXT_PRIMARY),
                ))
            })
            .collect()
    };
    let max_scroll = total_visual_lines.saturating_sub(usize::from(area.height));
    let scroll = scroll.min(max_scroll);
    Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .scroll((scroll.min(usize::from(u16::MAX)) as u16, 0))
        .style(Style::default().bg(background))
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

fn panel_block(title: &str, background: Color, focused: bool) -> Block<'static> {
    let border = if focused {
        palette::DEEPSEEK_SKY
    } else {
        palette::BORDER_COLOR
    };
    Block::default()
        .title(Line::from(Span::styled(
            format!(" {title} "),
            Style::default().fg(if focused {
                palette::TEXT_PRIMARY
            } else {
                palette::TEXT_MUTED
            }),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(background))
}

fn status_lines(session: &LoadedGameSession) -> Vec<String> {
    let mut lines = session.view.status.clone();
    lines.push(format!("Validation: {}", session.view.validation));
    if lines.is_empty() {
        lines.push(format!("Save revision {}", session.revision));
    }
    lines
}

fn scene_lines(session: &LoadedGameSession) -> Vec<String> {
    if session.game_id == "reconciliation-demo" {
        return reconciliation_scene_lines(session.language);
    }

    let mut lines = vec![session.view.scene_title.clone(), String::new()];
    lines.extend(fallback_lines(&session.view.scene));
    for panel in session
        .panels
        .iter()
        .filter(|panel| matches!(panel.id.as_str(), "scene" | "briefing" | "goal"))
    {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(localize_common_line(session.language, &panel.title));
        lines.extend(
            fallback_lines(&panel.body)
                .into_iter()
                .map(|line| localize_common_line(session.language, &line)),
        );
    }
    trim_trailing_blank_lines(&mut lines);
    lines
}

fn reconciliation_scene_lines(language: GameLanguage) -> Vec<String> {
    if language.is_chinese() {
        vec![
            "背景：你和绫波丽在东京靠细小的照顾慢慢靠近，也约定过：恐惧出现时要说出口，不要让它变成距离。".to_string(),
            "今晚她要离开，因为你的沉默让这个约定像是已经被打碎。".to_string(),
            "开场：雨挂在铁轨的灯光里。她离楼梯只差一步，你只有一次机会把真实的话说出来。".to_string(),
        ]
    } else {
        vec![
            "Background: You and Ayanami Rei grew close in Tokyo through small acts of care, and made a private promise: name fear before it becomes distance.".to_string(),
            "Tonight she is leaving because your silence made that promise feel broken.".to_string(),
            "Opening: rain hangs in the rail lights. She is almost at the stairs, and you have one chance to answer honestly.".to_string(),
        ]
    }
}

fn localize_common_line(language: GameLanguage, line: &str) -> String {
    if !language.is_chinese() {
        return line.to_string();
    }
    match line {
        "Scene" => "场景".to_string(),
        "Station Overpass" => "车站天桥".to_string(),
        "Goal" => "目标".to_string(),
        "Be honest before she leaves." => "在她离开前说出诚实的话。".to_string(),
        "Rain hangs in the rail lights. She is almost at the stairs." => {
            "雨挂在铁轨灯光里。她几乎已经走到楼梯口。".to_string()
        }
        _ => line.to_string(),
    }
}

fn items_lines(session: &LoadedGameSession) -> Vec<String> {
    if session.view.items.is_empty() {
        vec!["No carried or visible items.".to_string()]
    } else {
        session.view.items.clone()
    }
}

fn tasks_lines(session: &LoadedGameSession) -> Vec<String> {
    if session.view.tasks.is_empty() {
        vec!["No active game tasks.".to_string()]
    } else {
        session.view.tasks.clone()
    }
}

fn dialogue_lines(session: &LoadedGameSession, player_log: &[String]) -> Vec<String> {
    if player_log.is_empty() && session.revision == 0 {
        return vec![
            "Before we begin, what language do you want to play in?".to_string(),
            "English, Chinese, bilingual, or another preference are all fine.".to_string(),
        ];
    }

    let mut lines = plain_dialogue_lines(player_log);
    if lines.is_empty() {
        lines = plain_dialogue_lines(&session.view.dialogue);
    } else if !session.view.dialogue.is_empty() {
        lines.push(String::new());
        lines.extend(plain_dialogue_lines(&session.view.dialogue));
    }
    lines
}

fn plain_dialogue_lines(lines: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    for line in lines {
        append_plain_dialogue_body(&mut output, line);
    }
    trim_trailing_blank_lines(&mut output);
    output
}

fn choices_lines(session: &LoadedGameSession) -> Vec<String> {
    if session.view.choices.is_empty() {
        vec!["Type a natural action or ask for the current options.".to_string()]
    } else {
        session
            .view
            .choices
            .iter()
            .map(|choice| strip_choice_input(choice))
            .collect()
    }
}

fn strip_choice_input(choice: &str) -> String {
    choice
        .split_once(" [")
        .map_or_else(|| choice.to_string(), |(before, _)| before.to_string())
}

fn compact_info_lines(session: &LoadedGameSession) -> Vec<String> {
    let mut lines = status_lines(session);
    lines.push(String::new());
    lines.extend(tasks_lines(session));
    if !session.view.items.is_empty() {
        lines.push(String::new());
        lines.extend(
            session
                .view
                .items
                .iter()
                .map(|item| format!("Item: {item}")),
        );
    }
    lines
}

fn fallback_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn player_log(app: &App) -> Vec<String> {
    let mut lines = Vec::new();
    for cell in &app.history {
        match cell {
            HistoryCell::User { content } => append_dialogue_entry(&mut lines, "You", content),
            HistoryCell::Assistant { content, .. } => {
                append_dialogue_entry(&mut lines, "Game", content);
            }
            _ => {}
        }
    }
    if let Some(HistoryCell::Assistant { content, .. }) =
        app.active_cell.as_ref().and_then(|cell| {
            cell.entries()
                .iter()
                .rev()
                .find(|entry| matches!(entry, HistoryCell::Assistant { .. }))
        })
        && !content.trim().is_empty()
    {
        append_dialogue_entry(&mut lines, "Game", content);
    }
    lines
}

fn append_dialogue_entry(lines: &mut Vec<String>, speaker: &str, content: &str) {
    let body = plain_dialogue_body_lines(content);
    if body.is_empty() {
        return;
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    for (index, line) in body.into_iter().enumerate() {
        if index == 0 {
            lines.push(format!("{speaker}: {line}"));
        } else if line.is_empty() {
            push_dialogue_blank(lines);
        } else {
            lines.push(format!("  {line}"));
        }
    }
}

fn append_plain_dialogue_body(lines: &mut Vec<String>, content: &str) {
    for line in plain_dialogue_body_lines(content) {
        if line.is_empty() {
            push_dialogue_blank(lines);
        } else {
            lines.push(line);
        }
    }
}

fn plain_dialogue_body_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.trim().is_empty() {
            push_dialogue_blank(&mut lines);
            continue;
        }
        let mut emitted = false;
        for segment in raw_line.split(" --- ") {
            let Some(line) = plain_dialogue_line(segment) else {
                push_dialogue_blank(&mut lines);
                continue;
            };
            if is_dialogue_metadata_line(&line) {
                continue;
            }
            lines.push(line);
            emitted = true;
        }
        if !emitted && raw_line.trim() == "---" {
            push_dialogue_blank(&mut lines);
        }
    }
    trim_trailing_blank_lines(&mut lines);
    lines
}

fn plain_dialogue_line(raw: &str) -> Option<String> {
    let mut line = raw.trim();
    if line.is_empty() || is_markdown_rule(line) {
        return None;
    }

    while let Some(stripped) = line.strip_prefix('>') {
        line = stripped.trim_start();
    }
    line = line.trim_start_matches('#').trim_start();
    for bullet in ["- ", "* ", "+ "] {
        if let Some(stripped) = line.strip_prefix(bullet) {
            line = stripped.trim_start();
            break;
        }
    }
    if line.is_empty() || is_markdown_rule(line) {
        return None;
    }

    let line = line.replace("**", "").replace("__", "").replace('`', "");
    let line = compact_whitespace(&line);
    (!line.is_empty()).then_some(line)
}

fn is_markdown_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3
        && trimmed
            .chars()
            .all(|ch| matches!(ch, '-' | '*' | '_' | ' '))
}

fn is_dialogue_metadata_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("current beat")
        || lower.starts_with("advance condition")
        || lower.starts_with("action skills")
        || line.starts_with("当前节拍")
        || line.starts_with("推进条件")
        || line.starts_with("可选行动技能")
}

fn push_dialogue_blank(lines: &mut Vec<String>) {
    if !lines.is_empty() && lines.last().is_none_or(|line| !line.is_empty()) {
        lines.push(String::new());
    }
}

fn trim_trailing_blank_lines(lines: &mut Vec<String>) {
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
}

fn compact_whitespace(text: &str) -> String {
    let mut output = String::new();
    for part in text.split_whitespace() {
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(part);
    }
    output
}

fn renderable_from_plain_frame(frame: &AsciiArtFrame) -> RenderableArtFrame {
    RenderableArtFrame {
        ratio_cols: frame.ratio_cols,
        ratio_rows: frame.ratio_rows,
        lines: frame
            .lines
            .iter()
            .map(|line| {
                line.chars()
                    .map(|symbol| StyledArtCell {
                        symbol,
                        style: Style::default().fg(palette::TEXT_PRIMARY),
                    })
                    .collect()
            })
            .collect(),
    }
}

fn load_ascii_art_source(game_root: &Path, source: &AsciiArtSource) -> Option<RenderableArtFrame> {
    let path = resolve_art_path(game_root, &source.path)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let lines = parse_ansi_art_lines(&raw);
    if lines.is_empty() {
        return None;
    }
    Some(RenderableArtFrame {
        lines,
        ratio_cols: source.ratio_cols,
        ratio_rows: source.ratio_rows,
    })
}

fn resolve_art_path(game_root: &Path, source_path: &str) -> Option<PathBuf> {
    let raw = PathBuf::from(source_path);
    if raw.is_absolute() {
        return None;
    }
    if raw.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(game_root.join(raw))
}

fn parse_ansi_art_lines(raw: &str) -> Vec<Vec<StyledArtCell>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut style = Style::default();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\u{1b}' => {
                if chars.peek() == Some(&'[') {
                    let _ = chars.next();
                    let mut sequence = String::new();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            if next == 'm' {
                                apply_sgr_sequence(&sequence, &mut style);
                            }
                            break;
                        }
                        sequence.push(next);
                    }
                } else {
                    skip_unsupported_escape(&mut chars);
                }
            }
            '\n' => {
                if !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                }
            }
            '\r' => {}
            symbol => row.push(StyledArtCell { symbol, style }),
        }
    }

    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

fn skip_unsupported_escape<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    match chars.peek().copied() {
        Some(']') => {
            let _ = chars.next();
            let mut previous_was_escape = false;
            for next in chars.by_ref() {
                if next == '\u{7}' || (previous_was_escape && next == '\\') {
                    break;
                }
                previous_was_escape = next == '\u{1b}';
            }
        }
        Some('P' | '^' | '_' | 'X') => {
            let _ = chars.next();
            let mut previous_was_escape = false;
            for next in chars.by_ref() {
                if previous_was_escape && next == '\\' {
                    break;
                }
                previous_was_escape = next == '\u{1b}';
            }
        }
        Some(_) => {
            let _ = chars.next();
        }
        None => {}
    }
}

fn apply_sgr_sequence(sequence: &str, style: &mut Style) {
    let values = if sequence.trim().is_empty() {
        vec![Some(0)]
    } else {
        sequence
            .split(';')
            .map(|part| part.parse::<u16>().ok())
            .collect::<Vec<_>>()
    };

    let mut index = 0usize;
    while index < values.len() {
        match values[index] {
            Some(0) => {
                *style = Style::default();
                index += 1;
            }
            Some(1) => {
                *style = style.add_modifier(Modifier::BOLD);
                index += 1;
            }
            Some(22) => {
                *style = style.remove_modifier(Modifier::BOLD);
                index += 1;
            }
            Some(30..=37) => {
                *style = style.fg(ansi16_color(values[index].unwrap_or(39), false));
                index += 1;
            }
            Some(40..=47) => {
                *style = style.bg(ansi16_color(values[index].unwrap_or(49) - 10, false));
                index += 1;
            }
            Some(90..=97) => {
                *style = style.fg(ansi16_color(values[index].unwrap_or(99) - 60, true));
                index += 1;
            }
            Some(100..=107) => {
                *style = style.bg(ansi16_color(values[index].unwrap_or(109) - 70, true));
                index += 1;
            }
            Some(38) | Some(48) => {
                let is_foreground = values[index] == Some(38);
                if values.get(index + 1) == Some(&Some(2))
                    && let (Some(Some(r)), Some(Some(g)), Some(Some(b))) = (
                        values.get(index + 2),
                        values.get(index + 3),
                        values.get(index + 4),
                    )
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (u8::try_from(*r), u8::try_from(*g), u8::try_from(*b))
                {
                    let color = Color::Rgb(r, g, b);
                    *style = if is_foreground {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                    index += 5;
                    continue;
                }
                if values.get(index + 1) == Some(&Some(5))
                    && let Some(Some(value)) = values.get(index + 2)
                    && let Ok(indexed) = u8::try_from(*value)
                {
                    let color = Color::Indexed(indexed);
                    *style = if is_foreground {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                    index += 3;
                    continue;
                }
                index += 1;
            }
            Some(39) => {
                *style = style.fg(Color::Reset);
                index += 1;
            }
            Some(49) => {
                *style = style.bg(Color::Reset);
                index += 1;
            }
            _ => {
                index += 1;
            }
        }
    }
}

fn ansi16_color(code: u16, bright: bool) -> Color {
    match (code, bright) {
        (30, false) => Color::Black,
        (31, false) => Color::Red,
        (32, false) => Color::Green,
        (33, false) => Color::Yellow,
        (34, false) => Color::Blue,
        (35, false) => Color::Magenta,
        (36, false) => Color::Cyan,
        (37, false) => Color::Gray,
        (30, true) => Color::DarkGray,
        (31, true) => Color::LightRed,
        (32, true) => Color::LightGreen,
        (33, true) => Color::LightYellow,
        (34, true) => Color::LightBlue,
        (35, true) => Color::LightMagenta,
        (36, true) => Color::LightCyan,
        (37, true) => Color::White,
        _ => Color::Reset,
    }
}

fn scale_renderable_art_lines(
    lines: &[Vec<StyledArtCell>],
    width: u16,
    height: u16,
) -> Vec<Vec<StyledArtCell>> {
    let target_width = usize::from(width);
    let target_height = usize::from(height);
    if target_width == 0 || target_height == 0 || lines.is_empty() {
        return Vec::new();
    }

    let source_height = lines.len();
    let output_height = source_height.min(target_height);
    (0..output_height)
        .map(|row| {
            let source_row = row.saturating_mul(source_height) / output_height;
            scale_renderable_art_line(&lines[source_row], target_width)
        })
        .collect()
}

fn scale_renderable_art_line(line: &[StyledArtCell], target_width: usize) -> Vec<StyledArtCell> {
    if target_width == 0 {
        return Vec::new();
    }
    if line.len() <= target_width {
        return line.to_vec();
    }
    (0..target_width)
        .map(|col| {
            let source_col = col.saturating_mul(line.len()) / target_width;
            line[source_col]
        })
        .collect()
}

fn fixed_ratio_rect(area: Rect, ratio_cols: u16, ratio_rows: u16) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }
    let ratio_cols = u32::from(ratio_cols.max(1));
    let ratio_rows = u32::from(ratio_rows.max(1));
    let area_width = u32::from(area.width);
    let area_height = u32::from(area.height);

    let (width, height) =
        if area_width.saturating_mul(ratio_rows) > area_height.saturating_mul(ratio_cols) {
            let width = area_height
                .saturating_mul(ratio_cols)
                .saturating_div(ratio_rows)
                .max(1);
            (width.min(area_width), area_height)
        } else {
            let height = area_width
                .saturating_mul(ratio_rows)
                .saturating_div(ratio_cols)
                .max(1);
            (area_width, height.min(area_height))
        };

    Rect {
        x: area
            .x
            .saturating_add((area.width.saturating_sub(width as u16)) / 2),
        y: area
            .y
            .saturating_add((area.height.saturating_sub(height as u16)) / 2),
        width: width as u16,
        height: height as u16,
    }
}

fn fit_line(text: &str, width: usize) -> String {
    if width == 0 || UnicodeWidthStr::width(text) <= width {
        return text.to_string();
    }

    let ellipsis = "...";
    let target = width.saturating_sub(ellipsis.len());
    let mut output = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if used + ch_width > target {
            break;
        }
        output.push(ch);
        used += ch_width;
    }
    output.push_str(ellipsis);
    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use deepseek_game::render::{GameViewSnapshot, RenderPanel};
    use ratatui::buffer::Buffer;

    use super::*;

    fn loaded_session() -> LoadedGameSession {
        LoadedGameSession {
            game_root: PathBuf::from("game"),
            saves_root: PathBuf::from("game/saves"),
            driver_root: None,
            game_id: "test-game".to_string(),
            title: "Test Game".to_string(),
            save_id: "default".to_string(),
            revision: 4,
            driver_id: "driver".to_string(),
            driver_requirement: "0.1.0".to_string(),
            locked_driver_version: Some("0.1.0".to_string()),
            panels: Vec::<RenderPanel>::new(),
            view: GameViewSnapshot {
                revision: 4,
                scene_title: "Room".to_string(),
                scene: "A quiet room with a table.".to_string(),
                figure_title: "Speaker".to_string(),
                figure: "Mood: careful".to_string(),
                status: vec!["Player: You".to_string(), "Votes: 1 / 1".to_string()],
                items: vec!["key".to_string()],
                tasks: vec!["listen".to_string()],
                dialogue: vec!["Speaker: Wait.".to_string()],
                choices: vec!["1. Ask".to_string()],
                validation: "valid".to_string(),
                scene_art: None,
                figure_art: None,
                figure_art_source: None,
                figure_emotion: Some("neutral".to_string()),
                music: None,
            },
            action_skills: Vec::new(),
            skills: Vec::new(),
            warnings: Vec::new(),
            developer_mode: false,
            language: crate::game::GameLanguage::English,
        }
    }

    #[test]
    fn fixed_ratio_rect_stays_inside_area() {
        let area = Rect::new(2, 3, 40, 10);
        let fitted = fixed_ratio_rect(area, 4, 3);
        assert!(fitted.x >= area.x);
        assert!(fitted.y >= area.y);
        assert!(fitted.right() <= area.right());
        assert!(fitted.bottom() <= area.bottom());
    }

    #[test]
    fn game_console_renders_player_panels_without_coding_chrome() {
        let session = GameSession::Loaded(loaded_session());
        let widget = GameConsoleWidget::from_parts(
            Some(&session),
            vec!["You: I ask a question.".to_string()],
        );
        let area = Rect::new(0, 0, 90, 28);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf);
        let mut text = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                text.push_str(buf[(x, y)].symbol());
            }
        }

        assert!(text.contains("Test Game"));
        assert!(text.contains("Dialogue"));
        assert!(text.contains("Choices"));
        assert!(!text.contains("Plan"));
        assert!(!text.contains("model"));
        assert!(!text.contains("cost"));
    }

    #[test]
    fn dialogue_lines_render_plain_organized_text_without_markdown_markers() {
        let mut session = loaded_session();
        session.view.dialogue.clear();
        let lines = dialogue_lines(
            &session,
            &["Game: 好的，用中文继续。\n---\n## 车站天桥\n**绫波丽** [轻声]：「我没办法一直猜自己对你重不重要。」\n**当前节拍**：最后一级台阶".to_string()],
        );

        assert_eq!(
            lines,
            vec![
                "Game: 好的，用中文继续。".to_string(),
                String::new(),
                "车站天桥".to_string(),
                "绫波丽 [轻声]：「我没办法一直猜自己对你重不重要。」".to_string(),
            ]
        );
        let text = lines.join("\n");
        assert!(!text.contains("##"), "{text}");
        assert!(!text.contains("**"), "{text}");
        assert!(!text.contains("---"), "{text}");
        assert!(!text.contains("当前节拍"), "{text}");
    }

    #[test]
    fn game_console_renders_representative_terminal_sizes() {
        let session = GameSession::Loaded(loaded_session());
        for (width, height) in [(60, 20), (90, 28), (140, 40)] {
            let widget = GameConsoleWidget::from_parts(Some(&session), Vec::new());
            let area = Rect::new(0, 0, width, height);
            let mut buf = Buffer::empty(area);

            widget.render(area, &mut buf);

            let mut text = String::new();
            for y in 0..height {
                for x in 0..width {
                    text.push_str(buf[(x, y)].symbol());
                }
            }
            assert!(text.contains("Test Game"), "{width}x{height}");
            assert!(text.contains("Room"), "{width}x{height}");
        }
    }

    #[test]
    fn scale_art_lines_fits_actual_pane_size() {
        let source = (0..60)
            .map(|row| {
                (0..120)
                    .map(|col| StyledArtCell {
                        symbol: if (row + col) % 2 == 0 { '#' } else { '.' },
                        style: Style::default(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let scaled = scale_renderable_art_lines(&source, 37, 11);

        assert_eq!(scaled.len(), 11);
        assert!(scaled.iter().all(|line| line.len() <= 37));
    }

    #[test]
    fn load_ascii_art_source_preserves_ansi_color_and_rejects_escaping_paths() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let portrait_dir = temp.path().join("assets/portraits/ayanami-rei");
        fs::create_dir_all(&portrait_dir).expect("create portrait dir");
        fs::write(
            portrait_dir.join("Neutral.ansi"),
            "\u{1b}[31mABCD\u{1b}[0m\nEFGH\n",
        )
        .expect("write portrait art");

        let source = AsciiArtSource {
            path: "assets/portraits/ayanami-rei/Neutral.ansi".to_string(),
            emotion: "neutral".to_string(),
            label: "Neutral".to_string(),
            cols: 4,
            rows: 2,
            ratio_cols: 2,
            ratio_rows: 1,
        };
        let frame = load_ascii_art_source(temp.path(), &source).expect("load local portrait art");
        let rendered = frame
            .lines
            .iter()
            .map(|line| line.iter().map(|cell| cell.symbol).collect::<String>())
            .collect::<Vec<_>>();
        assert_eq!(rendered, vec!["ABCD".to_string(), "EFGH".to_string()]);
        assert_eq!(frame.lines[0][0].style.fg, Some(Color::Red));
        assert_eq!(frame.lines[1][0].style.fg, None);

        let escaping = AsciiArtSource {
            path: "../Neutral.ansi".to_string(),
            ..source
        };
        assert!(load_ascii_art_source(temp.path(), &escaping).is_none());
    }

    #[test]
    fn ansi_art_parser_skips_non_sgr_terminal_controls() {
        let rows =
            parse_ansi_art_lines("\u{1b}]0;window title\u{7}\u{1b}[?25l\u{1b}[31mAB\u{1b}[0m\n");
        let rendered = rows
            .iter()
            .map(|line| line.iter().map(|cell| cell.symbol).collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(rendered, vec!["AB".to_string()]);
        assert_eq!(rows[0][0].style.fg, Some(Color::Red));
    }

    #[test]
    fn wide_game_console_allocates_real_portrait_canvas() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let portrait_dir = temp.path().join("assets/portraits/test");
        fs::create_dir_all(&portrait_dir).expect("create portrait dir");
        let art = (0..20)
            .map(|_| "@".repeat(80))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(portrait_dir.join("Neutral.ansi"), art).expect("write portrait art");

        let mut loaded = loaded_session();
        loaded.game_root = temp.path().to_path_buf();
        loaded.view.figure_art_source = Some(AsciiArtSource {
            path: "assets/portraits/test/Neutral.ansi".to_string(),
            emotion: "neutral".to_string(),
            label: "Neutral".to_string(),
            cols: 80,
            rows: 20,
            ratio_cols: 2,
            ratio_rows: 1,
        });
        let session = GameSession::Loaded(loaded);
        let widget = GameConsoleWidget::from_parts(Some(&session), Vec::new());
        let area = Rect::new(0, 0, 140, 40);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf);

        let mut visible_art_cells = 0usize;
        for y in 0..area.height {
            for x in 0..area.width {
                if buf[(x, y)].symbol() == "@" {
                    visible_art_cells += 1;
                }
            }
        }
        assert!(
            visible_art_cells > 500,
            "portrait should render as a visible image, got {visible_art_cells} cells"
        );
    }
}
