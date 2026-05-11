# Game TUI Framework Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec summarizes the top-level maintainer-facing work contract for
`deepseek play`. The authoritative implementation plan remains
`docs/GAME_TUI_FRAMEWORK_SPEC.md`; this file exists so game-related prompts can
enter the same SPEC_files workflow.

Game work is intentionally split into separate spec systems:

- Reusable game driver and driver-runtime behavior:
  `SPEC_files/game_driver/`
- Individual game cartridges and their story/content/save contracts:
  `SPEC_files/games/`

Do not use this top-level file as the only spec for driver internals or for a
single game's content/mechanics. Use it for cross-cutting Game Console
integration, then update the relevant nested game-driver or per-game spec.

## Source Anchors

Primary code:

- `crates/cli/src/main.rs`
- `crates/tui/src/main.rs`
- `crates/tui/src/game.rs`
- `crates/tui/src/commands/game.rs`
- `crates/tui/src/tools/game.rs`
- `crates/game/`

Related code:

- `crates/tui/src/prompts.rs`
- `crates/tui/src/prompts/game_console.md`
- `crates/tui/src/tui/app.rs`
- `crates/tui/src/tui/ui.rs`
- `crates/tui/src/core/engine/tool_setup.rs`
- `crates/tui/src/tools/registry.rs`
- `crates/tui/src/tools/subagent/`
- `crates/tui/src/tui/subagent_routing.rs`

Canonical docs:

- `docs/GAME_TUI_FRAMEWORK_SPEC.md`
- `TAKEOVER_PROMPT.md`
- `examples/games/`
- `SPEC_files/game_driver/README.md`
- `SPEC_files/games/README.md`

Tests and fixtures:

- `crates/game/src/tests.rs`
- Game-related TUI/tool tests where present
- Example game fixtures under `examples/games/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/13_GAME_TUI_FRAMEWORK_SPEC.md
Goal:
Game behavior affected:
Current behavior:
Desired behavior:
Player/developer mode expectation:
Save or manifest implications:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- Game TUI is a TUI-owned Game Console scaffold, not a separate terminal app.
- V1 must use `GameSession` rather than adding `AppMode::Game`.
- `crates/game` is a pure Rust runtime crate for manifests, saves, lookup,
  render data, Starlark driver behavior, and commit behavior.
- `crates/game` must not depend on ratatui, the TUI, LLM client, shell/network,
  Python, or external runtime repos.
- Player mode uses a restricted game-safe tool profile.
- Player-mode prompt control is consolidated in
  `crates/tui/src/prompts/game_console.md`, composed by
  `crates/tui/src/prompts.rs` as the single Game Console prompt file.
- The Game Console prompt owns the control loop:
  `observe -> classify -> estimate -> constrain -> commit -> render`.
- Prompt priority is
  `controller > save invariants > action skill > driver skill > NPC proposal > storytelling style`.
- `[game]` config remains reserved until config loader and `/config` UI support
  it.

## Design Principles

- Keep game runtime logic isolated from TUI presentation and LLM execution.
- Player mode should expose only game-safe tools and scoped helpers.
- Keep the Game Console prompt surface consolidated; prefer one clear prompt
  file over stacked controller, guardrail, and mode fragments.
- Game and driver skills should carry cartridge/driver-specific policy only.
  Shared silent-tool, player-mode hiding, fact-gate, commit-order, and save
  invariant rules belong in the Game Console prompt.
- Manifest, save, and story-branch behavior must be deterministic and
  recoverable.
- Docs, localization/help, tests, and code must ship together for new commands
  or tools.

## Change Workflow

- Read `docs/GAME_TUI_FRAMEWORK_SPEC.md` before making game changes.
- Choose the nested spec system before coding:
  - `SPEC_files/game_driver/` for driver resolution, manifests, Starlark
    functions, driver skills, reusable roles, and driver-owned validation.
  - `SPEC_files/games/` for one cartridge's premise, content, action grammar,
    facts, saves, endings, and game-specific skills.
- Confirm whether the change belongs in `crates/game`, TUI presentation, tool
  profile, CLI command routing, docs, or examples.
- For Game Console prompt changes, update
  `crates/tui/src/prompts/game_console.md` and keep
  `docs/GAME_TUI_FRAMEWORK_SPEC.md`, this file, and affected nested
  game/driver specs synchronized.
- Update `TAKEOVER_PROMPT.md` only to stay synchronized with the authoritative
  spec.
- Add tests for manifest/save/tool behavior and update example game fixtures
  when format changes.

## Acceptance Criteria Checklist

- [ ] Game runtime remains pure Rust and dependency-isolated.
- [ ] `deepseek play` behavior routes through the supported TUI path.
- [ ] Player tool profile remains restricted.
- [ ] `crates/tui/src/prompts/game_console.md` remains the single merged Game
      Console prompt file unless this spec changes first.
- [ ] Controller prompt tests cover single injection and required player-mode
      invariants.
- [ ] Manifests, saves, and lookups are validated and tested.
- [ ] Authoritative docs and handoff prompt stay synchronized.

## Validation Gates

- `cargo test -p deepseek-game --all-features` if package naming supports it.
- `cargo test -p deepseek-tui game_prompt_injects_single_turn_controller`
- `cargo test -p deepseek-tui game_turn_controller_pins_commit_and_player_mode_invariants`
- Targeted `crates/game` tests.
- Targeted TUI game command/tool tests.
- Full workspace tests for cross-surface game changes.

## Risks

- Adding a separate runtime or event loop would violate the intended
  architecture.
- Unscoped game tools can expose normal coding capabilities to player mode.
- Save format changes can break existing games without migrations.
