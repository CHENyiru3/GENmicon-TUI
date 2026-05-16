mod approval;
mod chat;
mod composer;
mod footer;
mod game_console;
mod header;
mod menu;
// Some helpers (`shift`, `ctrl_alt`, `is_press`, etc.) are part of the
// public surface for issue #93's help overlay and future call sites; allow
// dead code rather than scattering `#[allow]` across every constructor.
#[allow(dead_code)]
pub mod key_hint;
// Phase 1 of #85: widget lands without a wire-up site so reviewers can
// evaluate the rendering in isolation. The follow-up PR plumbs it through
// the composer area in `ui.rs`. `pub mod` (vs the usual `pub use` pattern)
// keeps the unused-imports lint quiet until then.
pub mod agent_card;
pub mod pending_input_preview;
mod renderable;
pub mod tool_card;

#[cfg(test)]
pub(crate) use approval::compute_takeover_area;
pub use approval::{ApprovalWidget, ElevationWidget};
pub use chat::ChatWidget;
#[cfg(test)]
pub(crate) use chat::{apply_selection_to_line, pad_lines_to_bottom, should_render_empty_state};
pub use composer::ComposerWidget;
#[cfg(test)]
pub(crate) use composer::{
    COMPOSER_PANEL_HEIGHT, composer_height, composer_max_height, composer_min_input_rows,
    composer_top_padding, cursor_row_col, layout_input, placeholder_visual_lines, wrap_input_lines,
    wrap_text,
};
pub(crate) use composer::{SlashMenuEntry, slash_completion_hints};
pub use footer::{
    FooterProps, FooterToast, FooterWidget, footer_agents_chip, footer_working_label,
};
pub use game_console::{GameConsoleProps, GameConsoleWidget, game_console_scroll_bounds};
pub use header::{HeaderData, HeaderWidget};
pub use renderable::Renderable;

#[cfg(test)]
mod tests {
    use super::{
        ApprovalWidget, COMPOSER_PANEL_HEIGHT, ChatWidget, ComposerWidget, Renderable,
        SlashMenuEntry, apply_selection_to_line, composer_height, composer_max_height,
        composer_min_input_rows, composer_top_padding, compute_takeover_area, cursor_row_col,
        layout_input, pad_lines_to_bottom, placeholder_visual_lines, should_render_empty_state,
        slash_completion_hints, wrap_input_lines, wrap_text,
    };
    use crate::config::Config;
    use crate::localization::Locale;
    use crate::palette;
    use crate::tui::app::{App, ComposerDensity, TuiOptions};
    use crate::tui::history::{GenericToolCell, HistoryCell, ToolCell, ToolStatus};
    use crate::tui::scrolling::TranscriptScroll;
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::Style,
        text::{Line, Span},
    };
    use std::path::PathBuf;
    use unicode_width::UnicodeWidthStr;

    fn create_test_app() -> App {
        let options = TuiOptions {
            model: "deepseek-v4-flash".to_string(),
            workspace: PathBuf::from("."),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("."),
            memory_path: PathBuf::from("memory.md"),
            notes_path: PathBuf::from("notes.txt"),
            mcp_config_path: PathBuf::from("mcp.json"),
            use_memory: false,
            start_in_agent_mode: true,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
            game_session: None,
        };
        App::new(options, &Config::default())
    }

    #[test]
    fn pad_lines_to_bottom_noop_when_already_filled() {
        let mut lines = vec![Line::from("one"), Line::from("two")];
        pad_lines_to_bottom(&mut lines, 2);
        assert_eq!(lines, vec![Line::from("one"), Line::from("two")]);
    }

    #[test]
    fn pad_lines_to_bottom_prepends_empty_lines() {
        let mut lines = vec![Line::from("one"), Line::from("two")];
        pad_lines_to_bottom(&mut lines, 5);

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], Line::from(""));
        assert_eq!(lines[1], Line::from(""));
        assert_eq!(lines[2], Line::from(""));
        assert_eq!(lines[3], Line::from("one"));
        assert_eq!(lines[4], Line::from("two"));
    }

    #[test]
    fn pad_lines_to_bottom_noop_when_height_is_zero() {
        let mut lines = vec![Line::from("one")];
        pad_lines_to_bottom(&mut lines, 0);
        assert_eq!(lines, vec![Line::from("one")]);
    }

    // Cursor alignment tests

    #[test]
    fn cursor_basic_ascii() {
        // "hello" with cursor at various positions, width=10
        assert_eq!(cursor_row_col("hello", 0, 10), (0, 0));
        assert_eq!(cursor_row_col("hello", 3, 10), (0, 3));
        assert_eq!(cursor_row_col("hello", 5, 10), (0, 5));
    }

    #[test]
    fn cursor_at_wrap_boundary() {
        // "abcde" exactly fills width=5
        // Cursor at position 5 (after last char) should wrap to next line
        let (row, col) = cursor_row_col("abcde", 5, 5);
        assert_eq!(row, 1, "cursor at end of full line should wrap");
        assert_eq!(col, 0, "cursor should be at start of next line");
    }

    #[test]
    fn cursor_with_cjk_characters() {
        // "中" is a CJK character with width 2
        // "a中b" = 1 + 2 + 1 = 4 display width
        assert_eq!(cursor_row_col("a中b", 0, 10), (0, 0)); // before 'a'
        assert_eq!(cursor_row_col("a中b", 1, 10), (0, 1)); // after 'a', before '中'
        assert_eq!(cursor_row_col("a中b", 2, 10), (0, 3)); // after '中', before 'b'
        assert_eq!(cursor_row_col("a中b", 3, 10), (0, 4)); // after 'b'
    }

    #[test]
    fn cursor_cjk_at_wrap_boundary() {
        // width=5, input "abcd中" (4 + 2 = 6, CJK doesn't fit on line 1)
        // CJK should wrap to next line
        let lines = wrap_text("abcd中", 5);
        assert_eq!(lines, vec!["abcd", "中"]);

        // Cursor after CJK should be on row 1, col 2
        let (row, col) = cursor_row_col("abcd中", 5, 5);
        assert_eq!(row, 1);
        assert_eq!(col, 2);
    }

    #[test]
    fn cursor_with_combining_marks() {
        // "e\u0301" is 'e' with combining acute accent (é)
        // Display width is 1 (combining mark has width 0)
        let input = "e\u{0301}"; // é as e + combining acute
        assert_eq!(input.chars().count(), 2);

        // Cursor positions:
        // 0 = before 'e'
        // 1 = after 'e', before combining mark
        // 2 = after combining mark
        assert_eq!(cursor_row_col(input, 0, 10), (0, 0));
        assert_eq!(cursor_row_col(input, 1, 10), (0, 1));
        assert_eq!(cursor_row_col(input, 2, 10), (0, 1)); // combining mark has width 0
    }

    #[test]
    fn cursor_with_emoji() {
        // Many emojis are double-width
        let input = "a😀b";
        // Cursor at 2 (after emoji) should account for emoji width
        let (_row, col) = cursor_row_col(input, 2, 10);
        // Emoji width varies by system, but should be either 1 or 2
        assert!((2..=3).contains(&col), "col = {col}, expected 2 or 3");
    }

    #[test]
    fn cursor_with_emoji_zwj_sequence() {
        let input = "👨‍👩‍👧‍👦";
        let cursor = input.chars().count();
        let (row, col) = cursor_row_col(input, cursor, 10);
        assert_eq!(row, 0);
        assert_eq!(col, input.width());
    }

    #[test]
    fn cursor_with_newlines() {
        // "ab\ncd" with cursor moving through
        assert_eq!(cursor_row_col("ab\ncd", 0, 10), (0, 0)); // before 'a'
        assert_eq!(cursor_row_col("ab\ncd", 2, 10), (0, 2)); // after 'b', before '\n'
        assert_eq!(cursor_row_col("ab\ncd", 3, 10), (1, 0)); // after '\n', before 'c'
        assert_eq!(cursor_row_col("ab\ncd", 5, 10), (1, 2)); // after 'd'
    }

    #[test]
    fn wrap_input_lines_preserves_empty_lines() {
        let lines = wrap_input_lines("a\n\nb", 10);
        assert_eq!(lines, vec!["a", "", "b"]);
    }

    #[test]
    fn wrap_input_lines_trailing_newline() {
        let lines = wrap_input_lines("a\n", 10);
        assert_eq!(lines, vec!["a", ""]);
    }

    #[test]
    fn cursor_and_wrap_consistency() {
        // Ensure cursor_row_col is consistent with wrap_text
        // for various inputs
        let test_cases = vec![
            ("hello world", 5),
            ("abcdefghij", 3),
            ("中文测试", 6),
            ("a\nb\nc", 10),
        ];

        for (input, width) in test_cases {
            let lines = wrap_input_lines(input, width);
            let (cursor_row, _) = cursor_row_col(input, input.chars().count(), width);

            // Cursor at end should be on the last line (or wrapped past it)
            assert!(
                cursor_row <= lines.len(),
                "cursor_row={cursor_row} should be <= lines.len()={} for input={input:?}",
                lines.len()
            );
        }
    }

    #[test]
    fn slash_completion_hints_include_links_and_config() {
        let hints = slash_completion_hints("/", 128, &[], Locale::En);
        assert!(hints.iter().any(|hint| hint.name == "/config"));
        assert!(hints.iter().any(|hint| hint.name == "/links"));
    }

    #[test]
    fn slash_completion_hints_exclude_set_and_deepseek_commands() {
        let hints = slash_completion_hints("/", 128, &[], Locale::En);
        assert!(!hints.iter().any(|hint| hint.name == "/set"));
        assert!(!hints.iter().any(|hint| hint.name == "/deepseek"));
    }

    #[test]
    fn slash_completion_hints_include_skills() {
        let cached_skills = vec![
            ("search-files".to_string(), "Search files".to_string()),
            ("my-review".to_string(), "Review code".to_string()),
        ];
        let hints = slash_completion_hints("/", 128, &cached_skills, Locale::En);
        assert!(
            hints
                .iter()
                .any(|hint| hint.name == "/skill search-files" && hint.is_skill)
        );
        assert!(
            hints
                .iter()
                .any(|hint| hint.name == "/skill my-review" && hint.is_skill)
        );
    }

    #[test]
    fn slash_completion_hints_skills_match_prefix() {
        let cached_skills = vec![
            ("search-files".to_string(), "Search files".to_string()),
            ("my-review".to_string(), "Review code".to_string()),
        ];
        let hints = slash_completion_hints("/se", 128, &cached_skills, Locale::En);
        assert!(
            hints
                .iter()
                .any(|hint| hint.name == "/skill search-files" && hint.is_skill)
        );
        assert!(!hints.iter().any(|hint| hint.name == "/skill my-review"));
    }

    #[test]
    fn slash_completion_hints_complete_skill_argument_prefix() {
        let cached_skills = vec![
            ("search-files".to_string(), "Search files".to_string()),
            ("my-review".to_string(), "Review code".to_string()),
        ];
        let hints = slash_completion_hints("/skill my", 128, &cached_skills, Locale::En);
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].name, "/skill my-review");
        assert!(hints[0].is_skill);
    }

    #[test]
    fn slash_completion_hints_prioritize_game_action_skills_for_skill_command() {
        let cached_skills = vec![
            (
                "academic-writing-editor".to_string(),
                "Edit prose".to_string(),
            ),
            (
                "chat".to_string(),
                "Game action: Chat - say something".to_string(),
            ),
            (
                "reflection".to_string(),
                "Game action: Reflection - rethink".to_string(),
            ),
        ];
        let hints = slash_completion_hints("/skill", 128, &cached_skills, Locale::En);

        assert_eq!(hints[0].name, "/skill chat");
        assert_eq!(hints[1].name, "/skill reflection");
    }

    #[test]
    fn selection_style_uses_explicit_selection_text_role() {
        let line = Line::from(Span::styled(
            "hello world",
            Style::default().fg(palette::TEXT_PRIMARY),
        ));
        let selection_style = Style::default()
            .bg(palette::SELECTION_BG)
            .fg(palette::SELECTION_TEXT);

        let styled = apply_selection_to_line(&line, 0, 5, selection_style);
        assert_eq!(styled.len(), 2);
        assert_eq!(styled[0].content.as_ref(), "hello");
        assert_eq!(styled[0].style.fg, Some(palette::SELECTION_TEXT));
        assert_eq!(styled[0].style.bg, Some(palette::SELECTION_BG));
        assert_eq!(styled[1].content.as_ref(), " world");
    }

    #[test]
    fn composer_layout_helpers_stay_consistent() {
        let input = "line one wraps nicely\nline two wraps as well";
        let width = 16;
        let available_height = 6;
        let menu_lines = 2;

        let height = composer_height(
            input,
            width,
            available_height,
            menu_lines,
            ComposerDensity::Comfortable,
            true,
        );
        let has_panel = available_height >= 3 && width >= 12;
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
        let input_height_budget = usize::from(height)
            .saturating_sub(menu_lines)
            .saturating_sub(chrome_height)
            .max(1);
        let (visible, cursor_row, cursor_col) = layout_input(
            input,
            input.chars().count(),
            content_width,
            input_height_budget,
        );

        assert!(visible.len().saturating_add(menu_lines) <= usize::from(height));
        assert!(!visible.is_empty());
        assert!(cursor_row < visible.len());
        assert!(cursor_col < content_width.max(1));
        assert!(height >= 5);
    }

    #[test]
    fn composer_height_prefers_panel_shape_when_space_allows() {
        let height = composer_height("", 40, 8, 0, ComposerDensity::Comfortable, true);
        assert_eq!(height, 5);
    }

    #[test]
    fn composer_height_skips_panel_chrome_when_border_disabled() {
        let with_border = composer_height("", 40, 8, 0, ComposerDensity::Comfortable, true);
        let without_border = composer_height("", 40, 8, 0, ComposerDensity::Comfortable, false);

        assert_eq!(with_border, 5);
        assert_eq!(without_border, 1);
        assert!(without_border < with_border);
    }

    #[test]
    fn composer_density_changes_min_rows_and_height_cap() {
        assert_eq!(composer_min_input_rows(ComposerDensity::Compact), 2);
        assert_eq!(composer_min_input_rows(ComposerDensity::Spacious), 4);
        assert!(
            composer_max_height(ComposerDensity::Spacious)
                > composer_max_height(ComposerDensity::Compact)
        );
    }

    #[test]
    fn empty_composer_cursor_matches_placeholder_padding() {
        let mut app = create_test_app();
        // Pin density so the test is independent of any loaded user settings.
        app.composer_density = ComposerDensity::Comfortable;
        let slash_menu_entries = Vec::<SlashMenuEntry>::new();
        let mention_menu_entries = Vec::<String>::new();
        let widget = ComposerWidget::new(&app, 5, &slash_menu_entries, &mention_menu_entries);

        // Use a wide area so the placeholder fits on one line (no wrapping).
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 5,
        };

        // inner_area: {x:1, y:1, w:38, h:3}  (borders shrink by 1 each side)
        // input_rows_budget = 3
        // placeholder_visual_lines(38) = 1  (placeholder is 22 chars, fits in 38)
        // top_padding = 3 - clamp(1, 1, 3) = 2
        // cursor_x = 0 + (1-0) + 0 = 1
        // cursor_y = 0 + (1-0) + (2+0) = 3
        assert_eq!(widget.cursor_pos(area), Some((1, 3)));
    }

    #[test]
    fn empty_composer_cursor_accounts_for_placeholder_wrapping() {
        let mut app = create_test_app();
        app.composer_density = ComposerDensity::Comfortable;
        let slash_menu_entries = Vec::<SlashMenuEntry>::new();
        let mention_menu_entries = Vec::<String>::new();
        let widget = ComposerWidget::new(&app, 5, &slash_menu_entries, &mention_menu_entries);

        // Narrow area forces the placeholder to wrap.
        let area = Rect {
            x: 0,
            y: 0,
            width: 14,
            height: 5,
        };

        // inner_area: {x:1, y:1, w:12, h:3}
        // input_rows_budget = 3
        // placeholder_visual_lines(12) = 2  ("Write a task" / " or use /.")
        // top_padding = 3 - clamp(2, 1, 3) = 1
        // cursor_x = 0 + (1-0) + 0 = 1
        // cursor_y = 0 + (1-0) + (1+0) = 2
        assert_eq!(placeholder_visual_lines(12), 2);
        assert_eq!(widget.cursor_pos(area), Some((1, 2)));
    }

    #[test]
    fn slash_menu_open_locks_composer_height_against_match_count_changes() {
        // Repro for the Windows 10 PowerShell + WSL feedback: typing
        // through a slash command shrinks the matched-entry list, which
        // used to shrink the composer height — and shrinking the
        // composer forces the chat area above to repaint every
        // keystroke.  With the height lock, the desired height returned
        // for a 5-match menu and a 1-match menu must be identical so
        // the layout stays stable for the lifetime of the slash session.
        let mut app = create_test_app();
        app.composer_density = ComposerDensity::Comfortable;
        app.input = "/skill".to_string();

        let many_matches: Vec<SlashMenuEntry> = (0..5)
            .map(|i| SlashMenuEntry {
                name: format!("/skill{i}"),
                description: String::new(),
                is_skill: false,
            })
            .collect();
        let one_match = vec![SlashMenuEntry {
            name: "/skill".to_string(),
            description: String::new(),
            is_skill: false,
        }];
        let no_matches = Vec::<SlashMenuEntry>::new();

        let widget_many = ComposerWidget::new(&app, 9, &many_matches, &[]);
        let widget_one = ComposerWidget::new(&app, 9, &one_match, &[]);
        let widget_none = ComposerWidget::new(&app, 9, &no_matches, &[]);

        // Fixed worst-case envelope while the slash menu is open.
        let height_many = widget_many.desired_height(40);
        let height_one = widget_one.desired_height(40);
        assert_eq!(
            height_many, height_one,
            "slash menu height must not jitter as the matched-entry count changes"
        );

        // Sanity: closing the slash menu (no matches) lets the panel
        // collapse back to a tight composer — we only want to lock
        // height *while* the menu is open.
        let height_none = widget_none.desired_height(40);
        assert!(
            height_none < height_many,
            "with the menu closed the composer should release the reserved rows; got {height_none} vs locked {height_many}"
        );
    }

    #[test]
    fn empty_composer_cursor_uses_full_area_when_border_disabled() {
        let mut app = create_test_app();
        app.composer_density = ComposerDensity::Comfortable;
        app.composer_border = false;
        let slash_menu_entries = Vec::<SlashMenuEntry>::new();
        let mention_menu_entries = Vec::<String>::new();
        let widget = ComposerWidget::new(&app, 3, &slash_menu_entries, &mention_menu_entries);

        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 3,
        };

        assert_eq!(widget.cursor_pos(area), Some((0, 2)));
    }

    #[test]
    fn localized_composer_placeholders_render_at_narrow_widths() {
        for locale in [Locale::Ja, Locale::ZhHans, Locale::PtBr] {
            let mut app = create_test_app();
            app.ui_locale = locale;
            app.composer_density = ComposerDensity::Comfortable;
            let slash_menu_entries = Vec::<SlashMenuEntry>::new();
            let mention_menu_entries = Vec::<String>::new();
            let widget = ComposerWidget::new(&app, 5, &slash_menu_entries, &mention_menu_entries);
            let area = Rect {
                x: 0,
                y: 0,
                width: 18,
                height: 5,
            };
            let mut buf = Buffer::empty(area);

            widget.render(area, &mut buf);
            let Some((cursor_x, cursor_y)) = widget.cursor_pos(area) else {
                panic!("localized composer should expose cursor position");
            };

            assert!(cursor_x < area.width, "{locale:?} cursor x overflow");
            assert!(cursor_y < area.height, "{locale:?} cursor y overflow");
        }
    }

    #[test]
    fn composer_top_padding_uses_clamp() {
        // content_lines=0 is clamped to 1
        assert_eq!(composer_top_padding(0, 3), 2);
        // content_lines=1
        assert_eq!(composer_top_padding(1, 3), 2);
        // content_lines=3 fills the budget
        assert_eq!(composer_top_padding(3, 3), 0);
        // content_lines > budget is clamped
        assert_eq!(composer_top_padding(5, 3), 0);
    }

    #[test]
    fn empty_state_renders_only_without_transcript_activity() {
        let mut app = create_test_app();
        assert!(should_render_empty_state(&app));
        app.add_message(crate::tui::history::HistoryCell::User {
            content: "hello".to_string(),
        });
        assert!(!should_render_empty_state(&app));
    }

    /// Probe: confirm `cell.lines_with_motion` returns no Line whose total
    /// visual width exceeds the requested area width, even for pathological
    /// long single-line tool results.
    #[test]
    fn long_tool_result_lines_fit_requested_width() {
        let cell = HistoryCell::Tool(ToolCell::Generic(GenericToolCell {
            name: "todo_write".to_string(),
            status: ToolStatus::Success,
            input_summary: Some("items: <2 items>".to_string()),
            output: Some("hello world ".repeat(420)),
            prompts: None,
            spillover_path: None,
        }));
        for width in [40u16, 80, 111, 165] {
            let lines = cell.lines(width);
            for (idx, line) in lines.iter().enumerate() {
                let visual: usize = line
                    .spans
                    .iter()
                    .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                    .sum();
                assert!(
                    visual <= usize::from(width),
                    "line {idx} at width {width} has visual width {visual} > {width}"
                );
            }
        }
    }

    /// Regression: a long single-line tool result must not write any cells
    /// outside the chat content area (issue #36 — sidebar gutter bleed).
    ///
    /// We render `ChatWidget` into a buffer that is wider than the chat area
    /// (simulating the sidebar split) and assert every cell to the right of
    /// `chat_area` is still the default empty cell.
    #[test]
    fn chat_widget_does_not_bleed_into_sidebar_for_long_tool_result() {
        // Reproduces the actual `todo_write` output shape: a status line,
        // a newline, then a pretty-printed JSON payload with long string
        // values. Run at several widths since the leak in the issue was
        // observed at ~165 cols.
        let cases: Vec<(u16, u16)> = vec![(80, 50), (120, 80), (165, 111), (200, 140)];
        for (total_width, chat_width) in cases {
            let mut app = create_test_app();
            let long_value: String = "hello world ".repeat(420);
            let json_payload = format!(
                "{{\n  \"items\": [\n    {{ \"id\": 1, \"content\": \"{long_value}\", \"status\": \"pending\" }}\n  ]\n}}"
            );
            let output = format!("Todo list updated (1 items, 0% complete)\n{json_payload}");
            app.add_message(HistoryCell::Tool(ToolCell::Generic(GenericToolCell {
                name: "todo_write".to_string(),
                status: ToolStatus::Success,
                input_summary: Some("todos: <1 items>".to_string()),
                output: Some(output),
                prompts: None,
                spillover_path: None,
            })));

            let height: u16 = 30;
            let chat_area = Rect {
                x: 0,
                y: 0,
                width: chat_width,
                height,
            };
            let full_area = Rect {
                x: 0,
                y: 0,
                width: total_width,
                height,
            };
            let mut buf = Buffer::empty(full_area);

            let widget = ChatWidget::new(&mut app, chat_area);
            widget.render(chat_area, &mut buf);

            // Every cell outside chat_area should remain at default. If the
            // widget bled, we'll see leftover symbols.
            let default_symbol = " ";
            for y in 0..height {
                for x in chat_width..total_width {
                    let cell = &buf[(x, y)];
                    let sym = cell.symbol();
                    assert!(
                        sym == default_symbol || sym.is_empty(),
                        "[{total_width}x{height}, chat={chat_width}] cell ({x},{y}) leaked content {sym:?} outside chat_area"
                    );
                }
            }
        }
    }

    #[test]
    fn chat_widget_uses_configured_surface_background() {
        let mut app = create_test_app();
        let custom = ratatui::style::Color::Rgb(26, 27, 38);
        app.ui_theme = app.ui_theme.with_background_color(custom);
        app.add_message(HistoryCell::Assistant {
            content: "ready".to_string(),
            streaming: false,
        });

        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 5,
        };
        let mut buf = Buffer::empty(area);
        let widget = ChatWidget::new(&mut app, area);
        widget.render(area, &mut buf);

        assert_eq!(buf[(area.x, area.y)].bg, custom);
        assert_eq!(
            buf[(area.x + area.width - 1, area.y + area.height - 1)].bg,
            custom
        );
    }

    /// Regression: when the transcript scrollbar is visible, the rightmost
    /// content column must remain readable (the scrollbar gets its own
    /// 1-column gutter rather than overdrawing chat content).
    #[test]
    fn chat_widget_reserves_scrollbar_gutter_when_scrollbar_visible() {
        let mut app = create_test_app();
        // Many short messages → forces the scrollbar to be visible.
        for i in 0..200 {
            app.add_message(HistoryCell::User {
                content: format!("user message {i}"),
            });
        }

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 8,
        };
        let mut buf = Buffer::empty(area);
        let widget = ChatWidget::new(&mut app, area);
        widget.render(area, &mut buf);

        // The rightmost column should host the scrollbar track/thumb.
        // The penultimate column should still hold normal content (a digit,
        // letter, or space — never the scrollbar glyph).
        let scrollbar_track = "│";
        let scrollbar_thumb = "┃";
        let mut scrollbar_seen = false;
        for y in 0..area.height {
            let last = buf[(area.width - 1, y)].symbol();
            let penult = buf[(area.width - 2, y)].symbol();
            if last == scrollbar_track || last == scrollbar_thumb {
                scrollbar_seen = true;
            }
            assert!(
                penult != scrollbar_track && penult != scrollbar_thumb,
                "scrollbar leaked into column {} (cell {:?}) at row {y}",
                area.width - 2,
                penult
            );
        }
        assert!(
            scrollbar_seen,
            "scrollbar should be visible for a long history"
        );
    }

    #[test]
    fn chat_widget_shows_jump_to_latest_button_when_scrolled_up() {
        let mut app = create_test_app();
        app.use_mouse_capture = true;
        for i in 0..80 {
            app.add_message(HistoryCell::User {
                content: format!("user message {i}"),
            });
        }
        app.viewport.transcript_scroll = TranscriptScroll::at_line(0);

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 8,
        };
        let mut buf = Buffer::empty(area);
        let widget = ChatWidget::new(&mut app, area);
        widget.render(area, &mut buf);

        let button = app
            .viewport
            .jump_to_latest_button_area
            .expect("button appears when transcript is not at tail");
        assert_eq!(button.width, 3);
        assert_eq!(button.height, 3);
        assert_eq!(buf[(button.x + 1, button.y + 1)].symbol(), "↓");
    }

    #[test]
    fn chat_widget_hides_jump_to_latest_button_at_tail() {
        let mut app = create_test_app();
        app.use_mouse_capture = true;
        for i in 0..80 {
            app.add_message(HistoryCell::User {
                content: format!("user message {i}"),
            });
        }
        app.viewport.transcript_scroll = TranscriptScroll::to_bottom();

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 8,
        };
        let _widget = ChatWidget::new(&mut app, area);
        assert!(
            app.viewport.jump_to_latest_button_area.is_none(),
            "button should hide while following the live tail"
        );
        assert!(app.viewport.transcript_scroll.is_at_tail());
    }

    /// Regression for issue #582: a resize event arriving while the
    /// engine is in `CoherenceState::RefreshingContext` (i.e. running
    /// a compaction summary call) must NOT leave the chat widget with
    /// an empty viewport. The user-reported symptom on Windows
    /// PowerShell is that the screen turns black on the maximize→
    /// windowed transition during a long task; the post-resize render
    /// must produce a populated frame regardless of the active
    /// coherence intervention. Pins the invariant from the renderer
    /// side; the actual ConHost size-stale fix lives in
    /// `tui::ui::run_tui` (the `Event::Resize` handler now forwards
    /// the event-reported dimensions to ratatui's viewport before the
    /// redraw).
    #[test]
    fn chat_widget_renders_cleanly_after_resize_during_refreshing_context() {
        use crate::core::coherence::CoherenceState;

        let mut app = create_test_app();
        for i in 0..30 {
            app.add_message(HistoryCell::User {
                content: format!("user message {i} during a long-running task"),
            });
        }

        // Pretend the engine is mid-compaction when the resize arrives.
        app.coherence_state = CoherenceState::RefreshingContext;

        // Drive the same shrink-then-grow cycle that maximize→windowed
        // transitions produce on Windows.
        for (width, height) in [(140u16, 40u16), (90, 28), (60, 20), (140, 40)] {
            app.handle_resize(width, height);
            let area = Rect {
                x: 0,
                y: 0,
                width,
                height,
            };
            let mut buf = Buffer::empty(area);
            let widget = ChatWidget::new(&mut app, area);
            widget.render(area, &mut buf);

            let mut non_empty = 0usize;
            for y in 0..height {
                for x in 0..width {
                    let sym = buf[(x, y)].symbol();
                    if sym != " " && !sym.is_empty() {
                        non_empty += 1;
                    }
                }
            }
            assert!(
                non_empty > 0,
                "resize-during-RefreshingContext at {width}x{height} produced an empty buffer; \
                 render path must not gate on coherence state (#582)"
            );
        }

        // The engine's coherence_state must survive a resize — it is
        // the engine's runtime decision, not a render-loop concern.
        // A future regression that bounced the state to `Healthy` on
        // resize would silently drop the "refreshing context" footer
        // chip while compaction is still in flight.
        assert_eq!(
            app.coherence_state,
            CoherenceState::RefreshingContext,
            "resize must not mutate engine-owned coherence_state"
        );
    }

    #[test]
    fn approval_takeover_clamps_to_short_terminal_height() {
        let request = crate::tui::approval::ApprovalRequest::new(
            "approval-1",
            "exec_shell",
            "Run git commit",
            &serde_json::json!({ "command": "git commit -m fix" }),
            "exec_shell:git commit",
        );
        let view = crate::tui::approval::ApprovalView::new(request.clone());
        let widget = ApprovalWidget::new(&request, &view);

        for area in [Rect::new(0, 0, 162, 17), Rect::new(0, 0, 39, 17)] {
            let card_area = compute_takeover_area(area);
            assert!(card_area.x >= area.x);
            assert!(card_area.y >= area.y);
            assert!(card_area.right() <= area.right());
            assert!(card_area.bottom() <= area.bottom());

            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
        }
    }

    /// Regression for issue #65: after `App::handle_resize`, the chat widget
    /// must produce a clean render at the new width — no stale wrapping,
    /// no panic, no content exceeding the requested width. Cycling through
    /// several widths (shrinks and grows) flushes any cached layout that
    /// fails to invalidate on resize.
    #[test]
    fn chat_widget_renders_cleanly_after_resize_cycle() {
        let mut app = create_test_app();
        // Add some long content that wraps differently at different widths.
        for i in 0..40 {
            app.add_message(HistoryCell::User {
                content: format!("user message {i} with enough text to wrap at 30 columns easily"),
            });
        }

        let widths_to_cycle = [120u16, 80, 40, 60, 100, 30];
        let height: u16 = 20;
        for width in widths_to_cycle {
            // Caller-side: simulate the resize handler invalidating caches.
            app.handle_resize(width, height);
            let area = Rect {
                x: 0,
                y: 0,
                width,
                height,
            };
            let mut buf = Buffer::empty(area);
            let widget = ChatWidget::new(&mut app, area);
            widget.render(area, &mut buf);

            // The render must produce at least some non-empty content for a
            // populated history at any reasonable width. This catches a class
            // of resize regressions where stale layout state leaves a blank
            // viewport after a width change.
            let mut non_empty = 0usize;
            for y in 0..height {
                for x in 0..width {
                    let sym = buf[(x, y)].symbol();
                    if sym != " " && !sym.is_empty() {
                        non_empty += 1;
                    }
                }
            }
            assert!(
                non_empty > 0,
                "render at {width}x{height} produced an empty buffer after resize"
            );
        }
    }

    /// Regression for issue #65: the transcript view cache must invalidate
    /// when width changes, so the same `App.history` re-wraps to the new
    /// width on the very next `ChatWidget::new` call.
    #[test]
    fn transcript_cache_invalidates_on_width_change() {
        let mut app = create_test_app();
        for i in 0..10 {
            app.add_message(HistoryCell::User {
                content: format!("a fairly long user message number {i} that needs to wrap"),
            });
        }

        let area_wide = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 20,
        };
        let area_narrow = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 20,
        };
        let mut buf_wide = Buffer::empty(area_wide);
        let widget_wide = ChatWidget::new(&mut app, area_wide);
        widget_wide.render(area_wide, &mut buf_wide);
        let wide_total_lines = app.viewport.transcript_cache.total_lines();

        // Without an explicit resize call, just shrinking the render area
        // should still trigger a cache rebuild because the cache keys on width.
        let mut buf_narrow = Buffer::empty(area_narrow);
        let widget_narrow = ChatWidget::new(&mut app, area_narrow);
        widget_narrow.render(area_narrow, &mut buf_narrow);
        let narrow_total_lines = app.viewport.transcript_cache.total_lines();

        assert!(
            narrow_total_lines > wide_total_lines,
            "narrow render should produce more wrapped lines (got {narrow_total_lines}, wide={wide_total_lines})"
        );
    }

    /// Issue #78 — perf bench for transcript scroll lag.
    ///
    /// Builds a 5000-entry history (mix of user / assistant / a few tool
    /// cells), then times `ChatWidget::new` at scroll offsets 0, 100, 500,
    /// and 2000 lines from the tail. The first call after history mutation
    /// pays the wrap cost; subsequent calls at different offsets should hit
    /// the per-cell cache and be ~constant time regardless of offset.
    ///
    /// Run with: `cargo test -p deepseek-tui --release bench_transcript_scroll
    /// -- --ignored --nocapture`
    #[test]
    #[ignore = "perf bench; run with --release"]
    fn bench_transcript_scroll_5000_messages() {
        use std::time::Instant;

        let mut app = create_test_app();
        // 5000 cells: alternating user / assistant with realistic-ish bodies
        // so wrapping cost is non-trivial. Every 50th cell is a (small)
        // generic tool cell, mirroring real transcripts.
        for i in 0..5000usize {
            let cell = if i % 50 == 49 {
                HistoryCell::Tool(ToolCell::Generic(GenericToolCell {
                    name: "grep_files".to_string(),
                    status: ToolStatus::Success,
                    input_summary: Some(format!("query: hit-{i}")),
                    output: Some(format!("found 12 matches in cell-{i}")),
                    prompts: None,
                    spillover_path: None,
                }))
            } else if i % 2 == 0 {
                HistoryCell::User {
                    content: format!(
                        "user message {i}: please review the changes in src/foo/bar.rs and \
                         tell me whether the new error handling looks reasonable"
                    ),
                }
            } else {
                HistoryCell::Assistant {
                    content: format!(
                        "Sure — looking at src/foo/bar.rs in cell {i}, the new error \
                         handling wraps each fallible call in `?` and propagates a \
                         typed `FooError`. That looks fine, but consider whether the \
                         `Display` impl needs to redact the inner path."
                    ),
                    streaming: false,
                }
            };
            app.add_message(cell);
        }

        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        };

        // Warm-up: first call after a full history build pays the wrap cost
        // for every cell. We don't time this — it's amortized across the
        // session and is not the user-visible problem.
        let _ = ChatWidget::new(&mut app, area);

        let visible = area.height as usize;
        // For each scroll target, snap the scroll position there and measure
        // a fresh ChatWidget::new(). The cache should hit for all unchanged
        // cells, so the time should be roughly constant regardless of
        // offset.
        for offset_from_tail in [0usize, 100, 500, 2000] {
            let total = app.viewport.transcript_cache.total_lines();
            let max_start = total.saturating_sub(visible);
            let target = max_start.saturating_sub(offset_from_tail);
            app.viewport.transcript_scroll =
                crate::tui::scrolling::TranscriptScroll::at_line(target);

            let iters: u32 = 10;
            let start = Instant::now();
            for _ in 0..iters {
                let _ = ChatWidget::new(&mut app, area);
            }
            let elapsed = start.elapsed();
            let per_call_us = elapsed.as_micros() / u128::from(iters);
            println!(
                "[bench_transcript_scroll] offset={offset_from_tail:>5} \
                 per_render={per_call_us:>6} \u{3bc}s  ({:>3} ms / {iters} iters)",
                elapsed.as_millis()
            );
        }

        // Streaming-delta scenario: append one assistant cell at the tail
        // and time a render. The cache should re-render only the new cell,
        // NOT every cell — even at deep scroll.
        for offset_from_tail in [0usize, 2000] {
            let total = app.viewport.transcript_cache.total_lines();
            let max_start = total.saturating_sub(visible);
            let target = max_start.saturating_sub(offset_from_tail);
            app.viewport.transcript_scroll =
                crate::tui::scrolling::TranscriptScroll::at_line(target);

            let iters: u32 = 10;
            let start = Instant::now();
            for i in 0..iters {
                app.add_message(HistoryCell::Assistant {
                    content: format!("delta {i}"),
                    streaming: false,
                });
                let _ = ChatWidget::new(&mut app, area);
            }
            let elapsed = start.elapsed();
            let per_call_us = elapsed.as_micros() / u128::from(iters);
            println!(
                "[bench_transcript_scroll] streaming offset={offset_from_tail:>5} \
                 per_render={per_call_us:>6} \u{3bc}s  ({:>3} ms / {iters} iters)",
                elapsed.as_millis()
            );
        }
    }
}
