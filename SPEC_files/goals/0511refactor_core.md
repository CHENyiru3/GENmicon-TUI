
# Modularize TUI Components Into tui-core

Status: Completed on 2026-05-11 through the planned staged extraction scope. Further redesign can
continue as follow-up work, but the leftover refactor slices tracked here are complete.

  ## Summary

  Modularize the UI as a staged crate extraction plus visual cleanup. Use crates/tui-core as the
  reusable rendering/component crate, and leave runtime orchestration, app state mutation, engine
  events, tool routing, and command handling in crates/tui.

  The goal is not to split every helper. The goal is to make the major UI surfaces easier to redesign
  safely: main shell, game console, modal views, shared panels, and status/footer composition.

  ## Current Progress

  - [x] Foundation stage:
      - `deepseek-tui` depends on `deepseek-tui-core`.
      - `deepseek-tui-core` owns the shared `Renderable` trait.
      - `crates/tui/src/tui/widgets/renderable.rs` remains as a compatibility re-export.
      - `deepseek-tui-core` has generic `theme`, `layout`, `panel`, `text`, and `list` modules.
      - `deepseek-tui-core` has focused tests for shell layout splitting, centered rects,
        panel chrome, Unicode text fitting/wrapping, and selected list-row rendering.
  - [x] Main shell extraction:
      - [x] Added a TUI-owned `MainShellProps` / `MainShellAreas` view model in
        `crates/tui/src/tui/shell.rs`.
      - [x] Replaced the top-level vertical region `Layout` block in
        `crates/tui/src/tui/ui.rs::render` with named shell areas.
      - [x] Added `deepseek-tui-core::layout::split_vertical_shell_with_preview`
        so the shell area calculation is reusable and tested.
      - [x] Moved the generic header renderer into `deepseek-tui-core::header`,
        while `crates/tui` keeps DeepSeek-specific mode labels and palette
        mapping in its wrapper.
      - [x] Moved the generic footer renderer into `deepseek-tui-core::footer`,
        while `crates/tui` keeps retry semantics, localized/app-specific chips,
        MCP/worked/status assembly, and status-item gating in its wrapper.
      - [x] Convert more shell rendering to plain props without `&App`:
        `GameConsoleWidget` now accepts `GameConsoleProps`; `crates/tui` builds those props from
        `App` at the render boundary.
  - [x] Widget split:
      - [x] Split approval/elevation widgets out of
        `crates/tui/src/tui/widgets/mod.rs` into
        `crates/tui/src/tui/widgets/approval.rs`.
      - [x] Split chat transcript widget out of
        `crates/tui/src/tui/widgets/mod.rs` into
        `crates/tui/src/tui/widgets/chat.rs`.
      - [x] Split composer widget out of
        `crates/tui/src/tui/widgets/mod.rs` into
        `crates/tui/src/tui/widgets/composer.rs`.
      - [x] Split shared popup menu/list helpers out of composer rendering into
        `crates/tui/src/tui/widgets/menu.rs`.
  - [x] Modal/view split:
      - `crates/tui/src/tui/views/stack.rs` owns view stack, modal events, and modal traits.
      - `crates/tui/src/tui/views/config.rs` owns `ConfigView`.
      - `crates/tui/src/tui/views/subagents.rs` owns `SubAgentsView` and subagent row assembly.
      - `crates/tui/src/tui/views/shell_control.rs`, `help.rs`, and `status_picker.rs` remain split
        by modal family.
  - [x] Game console redesign:
      - `deepseek-tui-core::art` owns ANSI art parsing, styled art cells/frames, art scaling, and
        fixed-ratio fitting.
      - `deepseek-tui-core::panel` owns generic text panel and art-or-text panel rendering, wrapping,
        and scroll-bound helpers.
      - `deepseek-tui-core::game_console` owns the shared wide/medium/narrow game-console area split,
        used by both rendering and scroll-bound calculation.
      - `crates/tui` keeps game session data, history-derived player log, asset path safety, language
        selection, and game-specific line assembly.

  Foundation validation:

  - `cargo test -p deepseek-tui-core`
  - `cargo test -p deepseek-tui`
    - Initial sandboxed run failed only because local mock-server tests could not bind OS ports.
    - Unrestricted rerun passed.
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo fmt --all -- --check`
  - `cargo test -p deepseek-tui game_console_scroll_up_moves_off_bottom_sentinel`

  Main shell slice validation:

  - `cargo test -p deepseek-tui-core split_vertical_shell_with_preview`
  - `cargo test -p deepseek-tui main_shell_areas`
  - `cargo test -p deepseek-tui smoke_boot_paints_composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo fmt --all -- --check`

  Header renderer extraction validation:

  - `cargo test -p deepseek-tui-core header`
  - `cargo test -p deepseek-tui header`
  - `cargo test -p deepseek-tui-core`
  - `cargo test -p deepseek-tui smoke_boot_paints_composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo fmt --all -- --check`

  Footer renderer extraction validation:

  - `cargo test -p deepseek-tui-core footer`
  - `cargo test -p deepseek-tui footer`
  - `cargo test -p deepseek-tui footer_priority_drop`
  - `cargo test -p deepseek-tui working_strip`
  - `cargo test -p deepseek-tui-core`
  - `cargo test -p deepseek-tui smoke_boot_paints_composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`

  Approval/elevation widget split validation:

  - `cargo test -p deepseek-tui approval_takeover_clamps_to_short_terminal_height`
  - `cargo test -p deepseek-tui smoke_boot_paints_composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`

  Composer widget split validation:

  - `cargo test -p deepseek-tui composer`
  - `cargo test -p deepseek-tui slash_completion_hints`
  - `cargo test -p deepseek-tui smoke_boot_paints_composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`
    - Initial sandboxed run failed only because local mock-server tests could not bind OS ports.
    - Unrestricted rerun passed.

  Chat transcript widget split validation:

  - `cargo test -p deepseek-tui tui::widgets::tests::`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`

  Shared popup menu/list helper split validation:

  - `cargo test -p deepseek-tui tui::widgets::menu::tests::`
  - `cargo test -p deepseek-tui composer`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`

  2026-05-11 completion slice validation:

  - `cargo test -p deepseek-tui-core art`
  - `cargo test -p deepseek-tui-core panel`
  - `cargo test -p deepseek-tui-core game_console`
  - `cargo test -p deepseek-tui-core`
  - `cargo test -p deepseek-tui tui::views::tests`
  - `cargo test -p deepseek-tui game_console`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo test -p deepseek-tui`
    - Initial sandboxed run failed only because local mock-server tests could not bind OS ports.
    - Unrestricted rerun passed.

  ## Key Changes

  - Add deepseek-tui-core as a dependency of deepseek-tui.
  - Move generic rendering primitives from crates/tui/src/tui/widgets into crates/tui-core:
      - Renderable trait
      - panel/chrome helpers
      - text wrapping and fitting helpers
      - reusable list/menu row rendering
      - common layout helpers for fixed-height footer/header/body/composer regions
  - Keep application-owned data in crates/tui:
      - App
      - GameConsoleState
      - engine/session/tool state
      - command palette actions and routed ViewEvents
  - Introduce prop/view-model structs at the crate boundary:
      - widgets in tui-core accept plain props, not &App
      - crates/tui builds props from App and routes events back
      - props should use owned display strings or borrowed slices where simple; avoid leaking app
        internals into tui-core
  - Redesign allowed, but only at the presentation layer:
      - improve spacing, panel hierarchy, and visual consistency
      - do not change keyboard semantics, command behavior, save/game state, or tool execution flow in
        the same pass

  ## Implementation Stages

  1. Foundation
      - Add ratatui, unicode-width, and unicode-segmentation dependencies to crates/tui-core.
      - Move Renderable into tui-core.
      - Add shared modules: theme, layout, panel, text, list.
      - Keep compatibility re-exports in crates/tui/src/tui/widgets during migration.
  2. Main Shell Extraction
      - Extract the top-level render layout from ui.rs into a shell view model and shell renderer.
      - ui.rs should decide what mode is active, build props, and call renderers.
      - Move generic header/footer rendering into tui-core; keep DeepSeek-specific labels and status
        assembly in crates/tui.
  3. Widget Split
      - Split the large widgets/mod.rs into focused modules:
          - chat transcript widget
          - composer widget
          - approval/elevation widgets
          - shared menu/list rendering
      - Move reusable rendering pieces to tui-core; keep app-specific composition in crates/tui.
  4. Modal/View Split
      - Split views/mod.rs by modal family:
          - view stack and event types
          - shell control
          - config view
          - subagent view
      - Only move view-independent modal chrome and list/table primitives to tui-core.
  5. Game Console Redesign
      - Treat the game console as the first redesigned surface.
      - Keep game session data in crates/tui, but move generic scene/figure/text/art panel rendering
        into tui-core.
      - Preserve the current scene art and portrait behavior while improving layout consistency between
        wide, medium, and narrow terminals.

  ## Test Plan

  - Add deepseek-tui-core unit tests for:
      - panel/chrome rendering
      - text fitting and wrapping
      - layout splitting at narrow/medium/wide widths
      - list/menu selection rendering
  - Keep or add crates/tui buffer-render tests for:
      - main shell at representative terminal sizes
      - composer cursor positioning
      - footer/status rendering
      - modal stack rendering
      - game console scene/figure panels
  - Run after each stage:
      - cargo test -p deepseek-tui-core
      - cargo test -p deepseek-tui
      - cargo clippy --workspace --all-targets --all-features
      - cargo fmt --all

  ## Assumptions

  - Use stable Rust only; no nightly features.
  - tui-core becomes a real reusable crate, but it must not depend on crates/tui.
  - Redesign is allowed for visual structure, not runtime behavior.
  - Migration should be staged with compatibility re-exports to avoid one massive review.
  - Existing dirty worktree changes must be preserved; implementation should avoid unrelated rewrites
