use std::time::Duration;

use crate::palette;
use crate::tui::app::App;
use crate::tui::history::HistoryCell;
use crate::tui::scrolling::TranscriptLineMeta;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};
use unicode_width::UnicodeWidthChar;

use super::Renderable;

const SEND_FLASH_DURATION: Duration = Duration::from_millis(500);
const JUMP_TO_LATEST_BUTTON_WIDTH: u16 = 3;
const JUMP_TO_LATEST_BUTTON_HEIGHT: u16 = 3;

pub struct ChatWidget {
    content_area: Rect,
    lines: Vec<Line<'static>>,
    scrollbar: Option<TranscriptScrollbar>,
    jump_to_latest_button: Option<Rect>,
    background: Color,
}

#[derive(Debug, Clone, Copy)]
struct TranscriptScrollbar {
    top: usize,
    visible: usize,
    total: usize,
}

impl ChatWidget {
    pub fn new(app: &mut App, area: Rect) -> Self {
        let content_area = area;
        let background = app.ui_theme.surface_bg;
        let visible_lines = content_area.height as usize;
        let render_options = app.transcript_render_options();

        if should_render_empty_state(app) {
            let lines = build_empty_state_lines(app, content_area);
            app.viewport.last_transcript_area = Some(content_area);
            app.viewport.last_transcript_top = 0;
            app.viewport.last_transcript_visible = visible_lines;
            app.viewport.last_transcript_total = 0;
            app.viewport.last_transcript_padding_top = 0;
            app.viewport.jump_to_latest_button_area = None;
            return Self {
                content_area,
                lines,
                scrollbar: None,
                jump_to_latest_button: None,
                background,
            };
        }

        // Per-cell revision caching (fix for issue #78):
        //
        // Every committed history cell carries its own revision counter in
        // `app.history_revisions`. The transcript cache compares each cell's
        // current revision against the previously rendered one, so unchanged
        // cells reuse their cached wrapped lines instead of being re-wrapped
        // every frame. This is the difference between O(history.len()) and
        // O(changed_cells) per render — and was the root cause of scroll lag
        // on long transcripts.
        //
        // The active in-flight cell (if any) is appended as the last cell so
        // its mutations show up at the live tail. Each entry inside the
        // active cell becomes a virtual cell at index `history.len() + i`,
        // matching `App::cell_at_virtual_index`. Active-cell entries share
        // the same `active_cell_revision` salt so any mutation in the active
        // cell forces only those rows to re-render — committed history rows
        // are unaffected.
        app.resync_history_revisions();
        let active_entries: &[HistoryCell] = app
            .active_cell
            .as_ref()
            .map_or(&[], |active| active.entries());

        let history_len = app.history.len();
        let has_collapsed = !app.collapsed_cells.is_empty();

        // Fast path: no collapsed cells — use original slices directly.
        if !has_collapsed {
            let mut cell_revisions: Vec<u64> =
                Vec::with_capacity(app.history.len() + active_entries.len());
            cell_revisions.extend_from_slice(&app.history_revisions);
            if !active_entries.is_empty() {
                let active_rev = app.active_cell_revision;
                for i in 0..active_entries.len() {
                    let salt = (i as u64).wrapping_add(1);
                    cell_revisions.push(
                        active_rev
                            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                            .wrapping_add(salt),
                    );
                }
            }
            // Build identity mapping: filtered index == original index.
            app.collapsed_cell_map = (0..app.history.len() + active_entries.len()).collect();

            let shards: [&[HistoryCell]; 2] = [&app.history, active_entries];
            app.viewport.transcript_cache.ensure_split(
                &shards,
                &cell_revisions,
                content_area.width.max(1),
                render_options,
            );
        } else {
            // Slow path: clone non-collapsed cells into filtered vecs so
            // collapsed cells are excluded from rendering. Build the
            // filtered→original index mapping.
            let mut filtered_cells: Vec<HistoryCell> =
                Vec::with_capacity(history_len + active_entries.len());
            let mut filtered_revs: Vec<u64> =
                Vec::with_capacity(history_len + active_entries.len());
            let mut filtered_to_original: Vec<usize> =
                Vec::with_capacity(history_len + active_entries.len());

            for (idx, cell) in app.history.iter().enumerate() {
                if app.collapsed_cells.contains(&idx) {
                    continue;
                }
                filtered_cells.push(cell.clone());
                filtered_revs.push(app.history_revisions[idx]);
                filtered_to_original.push(idx);
            }

            if !active_entries.is_empty() {
                let active_rev = app.active_cell_revision;
                for (i, cell) in active_entries.iter().enumerate() {
                    let original_idx = history_len + i;
                    if app.collapsed_cells.contains(&original_idx) {
                        continue;
                    }
                    filtered_cells.push(cell.clone());
                    let salt = (i as u64).wrapping_add(1);
                    filtered_revs.push(
                        active_rev
                            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                            .wrapping_add(salt),
                    );
                    filtered_to_original.push(original_idx);
                }
            }

            app.collapsed_cell_map = filtered_to_original;

            let shards: [&[HistoryCell]; 1] = [&filtered_cells];
            app.viewport.transcript_cache.ensure_split(
                &shards,
                &filtered_revs,
                content_area.width.max(1),
                render_options,
            );
        }

        let total_lines = app.viewport.transcript_cache.total_lines();

        let line_meta = app.viewport.transcript_cache.line_meta();

        if app.viewport.pending_scroll_delta != 0 {
            app.viewport.transcript_scroll = app.viewport.transcript_scroll.scrolled_by(
                app.viewport.pending_scroll_delta,
                line_meta,
                visible_lines,
            );
            app.viewport.pending_scroll_delta = 0;
        }

        let max_start = total_lines.saturating_sub(visible_lines);
        // v0.8.11 hotfix: snapshot whether the user's prior scroll state
        // was *deliberately* tail BEFORE we resolve. `resolve_top` clamps
        // out-of-range `at_line(N)` to `to_bottom()` (e.g. when content
        // shrunk so `max_start < N`), and `scrolled_by` returns
        // `to_bottom()` when the whole transcript fits in one screen
        // even if the user just scrolled up. Either case would fool a
        // post-resolve `is_at_tail()` check into thinking the user is
        // tracking the tail and silently revoke `user_scrolled_during_
        // stream` — the next stream chunk would then yank them back to
        // bottom mid-read.
        let was_explicit_tail = app.viewport.transcript_scroll.is_at_tail();
        let (scroll_state, top) = app
            .viewport
            .transcript_scroll
            .resolve_top(line_meta, max_start);
        app.viewport.transcript_scroll = scroll_state;
        // If the user scrolled back to the live tail, the per-stream
        // "leave me alone" lock is over — new chunks should pin to bottom
        // again until they explicitly scroll up. Without this clear, content
        // piles up off-screen below the visible area and the view appears
        // frozen at the moment they returned to bottom.
        //
        // Only clear the lock when the user's INTENT was tail (their
        // stored state was already `to_bottom()` before resolve), AND
        // when the transcript actually has scrolling room to talk about
        // — if everything fits in one screen, "tail" is trivially true
        // and clearing here would yank the user back to bottom on the
        // next chunk even though they explicitly scrolled up.
        if was_explicit_tail && total_lines > visible_lines {
            app.user_scrolled_during_stream = false;
        }

        app.viewport.last_transcript_area = Some(content_area);
        app.viewport.last_transcript_top = top;
        app.viewport.last_transcript_visible = visible_lines;
        app.viewport.last_transcript_total = total_lines;
        app.viewport.last_transcript_padding_top = 0;
        let detail_target_cell = (!app.viewport.transcript_selection.is_active())
            .then(|| app.detail_cell_index_for_viewport(top, visible_lines, line_meta))
            .flatten();

        let end = (top + visible_lines).min(total_lines);
        let mut lines = if total_lines == 0 {
            vec![Line::from("")]
        } else {
            app.viewport.transcript_cache.lines()[top..end].to_vec()
        };

        // Brief flash highlight on the most recently sent user message.
        if !app.low_motion
            && let Some(send_at) = app.last_send_at
        {
            if send_at.elapsed() < SEND_FLASH_DURATION {
                apply_send_flash(&mut lines, top, &app.history, line_meta);
            } else {
                app.last_send_at = None;
            }
        }

        if let Some(target_cell) = detail_target_cell {
            apply_detail_target_highlight(&mut lines, top, target_cell, line_meta);
        }

        apply_selection(&mut lines, top, app);

        if app.viewport.transcript_scroll.is_at_tail() {
            app.viewport.last_transcript_padding_top = visible_lines.saturating_sub(lines.len());
            pad_lines_to_bottom(&mut lines, visible_lines);
        }

        let scrollbar = (total_lines > visible_lines && content_area.width > 1).then_some(
            TranscriptScrollbar {
                top,
                visible: visible_lines,
                total: total_lines,
            },
        );
        let jump_to_latest_button =
            if app.use_mouse_capture && !app.viewport.transcript_scroll.is_at_tail() {
                jump_to_latest_button_rect(content_area, scrollbar.is_some())
            } else {
                None
            };
        app.viewport.jump_to_latest_button_area = jump_to_latest_button;

        Self {
            content_area,
            lines,
            scrollbar,
            jump_to_latest_button,
            background,
        }
    }
}

impl Renderable for ChatWidget {
    fn render(&self, _area: Rect, buf: &mut Buffer) {
        // Use the passed render area, not self.content_area — those can
        // drift when layout changes (e.g. file-tree pane toggle), and
        // using the stale self.content_area is the root cause of text
        // bleed-through (#400). In debug builds, assert the two match to
        // catch future drift early.
        debug_assert_eq!(
            _area, self.content_area,
            "ChatWidget content_area drifted from render area: \
             content_area={:?} render_area={:?}",
            self.content_area, _area
        );

        let area = _area;

        // Repaint the full chat area with the deepseek-ink background each
        // frame. Ratatui's `Paragraph` only writes cells that contain text,
        // so cells the current frame's paragraph doesn't touch would
        // otherwise hold the *previous* frame's contents (the `:24Z`
        // timestamp-tail bleed-through reported in v0.8.5 testing). Using
        // `Clear` reset cells to terminal default, which read as a brown-
        // gray on most user setups; an explicit ink fill keeps the chat
        // area on-brand.
        Block::default()
            .style(Style::default().bg(self.background))
            .render(area, buf);

        let paragraph =
            Paragraph::new(self.lines.clone()).style(Style::default().bg(self.background));
        paragraph.render(area, buf);

        if let Some(scrollbar) = self.scrollbar {
            let scrollable_range = scrollbar.total.saturating_sub(scrollbar.visible);
            let mut state = ScrollbarState::new(scrollable_range)
                .position(scrollbar.top.min(scrollable_range))
                .viewport_content_length(scrollbar.visible);
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .track_style(Style::default().fg(palette::BORDER_COLOR))
                .thumb_symbol("┃")
                .thumb_style(Style::default().fg(palette::DEEPSEEK_SKY))
                .render(area, buf, &mut state);
        }

        if let Some(button_area) = self.jump_to_latest_button {
            render_jump_to_latest_button(button_area, buf, self.background);
        }
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}

fn jump_to_latest_button_rect(area: Rect, has_scrollbar: bool) -> Option<Rect> {
    if area.width < JUMP_TO_LATEST_BUTTON_WIDTH + u16::from(has_scrollbar)
        || area.height < JUMP_TO_LATEST_BUTTON_HEIGHT
    {
        return None;
    }

    let scrollbar_gutter = u16::from(has_scrollbar);
    Some(Rect {
        x: area
            .x
            .saturating_add(area.width)
            .saturating_sub(scrollbar_gutter)
            .saturating_sub(JUMP_TO_LATEST_BUTTON_WIDTH),
        y: area
            .y
            .saturating_add(area.height)
            .saturating_sub(JUMP_TO_LATEST_BUTTON_HEIGHT),
        width: JUMP_TO_LATEST_BUTTON_WIDTH,
        height: JUMP_TO_LATEST_BUTTON_HEIGHT,
    })
}

fn render_jump_to_latest_button(area: Rect, buf: &mut Buffer, background: Color) {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette::BORDER_COLOR))
        .style(Style::default().bg(background))
        .render(area, buf);

    let arrow_x = area.x.saturating_add(1);
    let arrow_y = area.y.saturating_add(1);
    buf[(arrow_x, arrow_y)].set_symbol("↓").set_style(
        Style::default()
            .fg(palette::DEEPSEEK_SKY)
            .add_modifier(Modifier::BOLD),
    );
}

pub(crate) fn pad_lines_to_bottom(lines: &mut Vec<Line<'static>>, height: usize) {
    if lines.len() >= height {
        return;
    }
    let padding = height.saturating_sub(lines.len());
    if padding == 0 {
        return;
    }

    let mut padded = Vec::with_capacity(height);
    padded.extend(std::iter::repeat_n(Line::from(""), padding));
    padded.append(lines);
    *lines = padded;
}

fn apply_selection(lines: &mut [Line<'static>], top: usize, app: &App) {
    let Some((start, end)) = app.viewport.transcript_selection.ordered_endpoints() else {
        return;
    };

    let selection_style = Style::default()
        .bg(app.ui_theme.selection_bg)
        .fg(palette::SELECTION_TEXT);

    for (idx, line) in lines.iter_mut().enumerate() {
        let line_index = top + idx;
        if line_index < start.line_index || line_index > end.line_index {
            continue;
        }

        let (col_start, col_end) = if start.line_index == end.line_index {
            (start.column, end.column)
        } else if line_index == start.line_index {
            (start.column, usize::MAX)
        } else if line_index == end.line_index {
            (0, end.column)
        } else {
            (0, usize::MAX)
        };

        if col_start == 0 && col_end == usize::MAX {
            for span in &mut line.spans {
                span.style = span.style.patch(selection_style);
            }
            continue;
        }

        line.spans = apply_selection_to_line(line, col_start, col_end, selection_style);
    }
}

fn apply_detail_target_highlight(
    lines: &mut [Line<'static>],
    top: usize,
    target_cell: usize,
    line_meta: &[TranscriptLineMeta],
) {
    let highlight_bg = Color::Reset;
    for (idx, line) in lines.iter_mut().enumerate() {
        let line_index = top + idx;
        if let Some(TranscriptLineMeta::CellLine { cell_index, .. }) = line_meta.get(line_index)
            && *cell_index == target_cell
        {
            for span in &mut line.spans {
                span.style = span.style.bg(highlight_bg);
            }
        }
    }
}

/// Apply a brief background tint to the last user message's visible lines.
fn apply_send_flash(
    lines: &mut [Line<'static>],
    top: usize,
    history: &[HistoryCell],
    line_meta: &[TranscriptLineMeta],
) {
    // Find the last User cell index.
    let last_user_cell = history
        .iter()
        .rposition(|cell| matches!(cell, HistoryCell::User { .. }));
    let Some(target_cell) = last_user_cell else {
        return;
    };

    let flash_bg = Color::Rgb(30, 40, 55); // subtle dark-blue tint

    for (idx, line) in lines.iter_mut().enumerate() {
        let line_index = top + idx;
        if let Some(TranscriptLineMeta::CellLine { cell_index, .. }) = line_meta.get(line_index)
            && *cell_index == target_cell
        {
            for span in &mut line.spans {
                span.style = span.style.bg(flash_bg);
            }
        }
    }
}

pub(crate) fn apply_selection_to_line(
    line: &Line<'static>,
    col_start: usize,
    col_end: usize,
    selection_style: Style,
) -> Vec<Span<'static>> {
    let mut result = Vec::with_capacity(line.spans.len().saturating_add(2));
    let mut current_col = 0usize;

    for span in &line.spans {
        let span_text: &str = span.content.as_ref();
        let span_width = text_display_width(span_text);
        let span_end = current_col.saturating_add(span_width);

        if span_end <= col_start || current_col >= col_end {
            result.push(span.clone());
        } else if current_col >= col_start && span_end <= col_end {
            result.push(Span::styled(
                span.content.clone(),
                span.style.patch(selection_style),
            ));
        } else {
            let mut before = String::new();
            let mut selected = String::new();
            let mut after = String::new();
            let mut ch_col = current_col;

            for ch in span_text.chars() {
                let ch_width = char_display_width(ch);
                let ch_start = ch_col;
                let ch_end = ch_col.saturating_add(ch_width);
                if ch_end <= col_start {
                    before.push(ch);
                } else if ch_start >= col_end {
                    after.push(ch);
                } else {
                    selected.push(ch);
                }
                ch_col = ch_end;
            }

            if !before.is_empty() {
                result.push(Span::styled(before, span.style));
            }
            if !selected.is_empty() {
                result.push(Span::styled(selected, span.style.patch(selection_style)));
            }
            if !after.is_empty() {
                result.push(Span::styled(after, span.style));
            }
        }

        current_col = span_end;
    }

    result
}

fn text_display_width(text: &str) -> usize {
    text.chars().map(char_display_width).sum()
}

fn char_display_width(ch: char) -> usize {
    if ch == '\t' {
        4
    } else {
        UnicodeWidthChar::width(ch).unwrap_or(0).max(1)
    }
}

pub(crate) fn should_render_empty_state(app: &App) -> bool {
    app.history.is_empty() && !app.is_loading && !app.is_compacting
}

fn build_empty_state_lines(app: &App, area: Rect) -> Vec<Line<'static>> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let workspace_name = app
        .workspace
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| app.workspace.to_string_lossy().into_owned());
    let body_width = usize::from(area.width.saturating_sub(8).clamp(24, 72));
    let left_padding = usize::from(area.width.saturating_sub(body_width as u16) / 2);
    let inset = " ".repeat(left_padding);

    let body = vec![
        Line::from(Span::styled(
            format!("{inset}DeepSeek TUI"),
            Style::default().fg(palette::DEEPSEEK_BLUE).bold(),
        )),
        Line::from(Span::styled(
            format!("{inset}{workspace_name}  ·  {}", app.model),
            Style::default().fg(palette::TEXT_MUTED),
        )),
    ];

    let top_padding = usize::from(area.height.saturating_sub(body.len() as u16) / 3);
    let mut lines = Vec::new();
    for _ in 0..top_padding {
        lines.push(Line::from(""));
    }
    lines.extend(body);
    lines
}
