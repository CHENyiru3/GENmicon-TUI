# TUI App Runtime Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The TUI app runtime owns the interactive terminal experience: layout, transcript,
composer, keyboard handling, palettes, pickers, approvals, streaming display,
session views, and user-facing state transitions.

## Source Anchors

Primary code:

- `crates/tui/src/tui/app.rs`
- `crates/tui/src/tui/ui.rs`
- `crates/tui/src/tui/mod.rs`
- `crates/tui/src/main.rs`

Related code:

- `crates/tui/src/tui/transcript.rs`
- `crates/tui/src/tui/live_transcript.rs`
- `crates/tui/src/tui/command_palette.rs`
- `crates/tui/src/tui/slash_menu.rs`
- `crates/tui/src/tui/keybindings.rs`
- `crates/tui/src/tui/model_picker.rs`
- `crates/tui/src/tui/provider_picker.rs`
- `crates/tui/src/tui/session_picker.rs`
- `crates/tui/src/tui/subagent_routing.rs`
- `crates/tui/src/tui/tool_routing.rs`
- `crates/tui/src/tui/ui_text.rs`

Canonical docs:

- `docs/ARCHITECTURE.md`
- `docs/KEYBINDINGS.md`
- `docs/ACCESSIBILITY.md`
- `docs/MODES.md`

Tests and fixtures:

- `crates/tui/tests/`
- `crates/tui/src/tui/*` unit tests where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/02_TUI_APP_RUNTIME_SPEC.md
Goal:
Screen or interaction affected:
Current behavior:
Desired behavior:
Keyboard/mouse expectations:
Accessibility or localization needs:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- The TUI is ratatui-based and remains the live end-user runtime.
- It renders user/assistant transcript state, DeepSeek thinking blocks, tool
  calls, approvals, job routing, pickers, slash commands, and status surfaces.
- UI state routes through `AppAction` and app state in `tui/app.rs`, with event
  handling and rendering spread through `tui/ui.rs` and focused modules.

## Design Principles

- Prioritize keyboard-driven flows that are predictable under long sessions.
- Keep status visible without hiding the transcript or composer.
- Do not add user-facing copy in only one language surface when localization is
  required.
- Avoid UI behavior that depends on terminal size assumptions without fallback.

## Change Workflow

- Identify whether the change is rendering, state, input routing, command
  dispatch, or engine event handling.
- Update `docs/KEYBINDINGS.md` for new or changed keyboard behavior.
- Update `docs/ACCESSIBILITY.md` for focus, contrast, navigation, or screen
  reader relevant changes.
- Add tests around pure state transitions where possible; use PTY/mock-LLM tests
  for end-to-end behavior.

## Acceptance Criteria Checklist

- [ ] Interaction works from the canonical `deepseek` entry point.
- [ ] State changes are visible and reversible where expected.
- [ ] Terminal resizing and narrow layouts do not corrupt the UI.
- [ ] Relevant help, keybinding, and localized text are updated.
- [ ] Regression coverage or a manual TUI smoke test is recorded.

## Validation Gates

- Targeted TUI tests under `crates/tui/tests/`.
- `cargo test -p deepseek-tui --all-features`.
- Manual smoke test for visual or keyboard-heavy changes.

Use full workspace gates before merge for shared app-state changes.

## Risks

- UI changes can silently break one-shot, review, apply, eval, or server startup
  paths if they assume interactive state.
- Transcript rendering bugs are often only visible with streaming, thinking
  blocks, long tool output, or resumed sessions.
