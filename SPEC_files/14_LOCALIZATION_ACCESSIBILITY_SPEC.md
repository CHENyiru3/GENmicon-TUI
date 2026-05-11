# Localization And Accessibility Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns user-facing text quality, localization consistency, keybinding
documentation, accessibility expectations, and terminal usability across
languages and display environments.

## Source Anchors

Primary code:

- `crates/tui/src/localization.rs`
- `crates/tui/src/tui/ui_text.rs`
- `crates/tui/src/commands/mod.rs`
- `crates/tui/src/tui/keybindings.rs`

Related code:

- `crates/tui/src/tui/command_palette.rs`
- `crates/tui/src/tui/slash_menu.rs`
- `crates/tui/src/deepseek_theme.rs`
- `crates/tui/src/palette.rs`
- `crates/tui/src/tui/color_compat.rs`

Canonical docs:

- `docs/LOCALIZATION.md`
- `docs/ACCESSIBILITY.md`
- `docs/KEYBINDINGS.md`
- `README.md`
- `README.zh-CN.md`

Tests and fixtures:

- Localization/unit tests where present
- TUI rendering tests where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/14_LOCALIZATION_ACCESSIBILITY_SPEC.md
Goal:
Text/UI/accessibility surface affected:
Current behavior:
Desired behavior:
Languages affected:
Keyboard or screen-reader expectations:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- UI localization supports `en`, `ja`, `zh-Hans`, and `pt-BR` with
  auto-detection.
- Command text uses `MessageId` entries and localized UI text surfaces.
- Keybindings and accessibility expectations are documented separately.

## Design Principles

- User-facing copy should be clear, concise, and consistent with command/help
  behavior.
- New user-visible text should go through localization surfaces instead of
  being hard-coded ad hoc.
- Keyboard-only operation is a first-class interaction path.
- Terminal color and layout choices must remain usable in constrained
  environments.

## Change Workflow

- Identify all user-facing strings introduced or changed.
- Update localization entries, command metadata, help text, and docs together.
- Update keybinding docs for new shortcuts.
- Check narrow terminal and non-color-friendly behavior for visual changes.

## Acceptance Criteria Checklist

- [ ] New user-facing text is localized or intentionally documented as internal.
- [ ] Command palette, slash menu, help, and docs stay consistent.
- [ ] Keyboard behavior is documented and testable.
- [ ] Accessibility docs are updated for focus, contrast, or navigation changes.
- [ ] Text fits expected terminal widths without corrupting layout.

## Validation Gates

- Targeted localization tests.
- Targeted TUI rendering or keybinding tests.
- Manual smoke test for layout-heavy UI copy changes.

## Risks

- Hard-coded strings create incomplete localizations.
- Help text can drift from command behavior.
- Long translated strings can break narrow terminal layouts.
