# Driver Script Functions Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns deterministic driver functions declared in `driver.toml` and run
through the constrained script runtime.

## Source Anchors

Primary code:

- `crates/game/src/script.rs`
- `crates/game/src/driver.rs`
- `crates/tui/src/tools/game.rs`

Example scripts:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/scripts/affection.star`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/scripts/deliberation.star`

## Current Behavior

- Driver functions must be listed under `[functions.<name>]` in `driver.toml`.
- Each function names a script file, function symbol, and `mutates` flag.
- V1 functions are deterministic helpers; they do not directly write saves.
- `game_run_driver` can call declared functions only.

## Design Principles

- Driver functions calculate bounded mechanics, validation, scoring, or
  structured proposals.
- The main game turn remains responsible for narration and final commit.
- Undeclared functions must fail closed.
- Scripts are not a general shell, network, or file API.

## Acceptance Criteria Checklist

- [ ] Every callable function is declared in `driver.toml`.
- [ ] Undeclared function calls fail without side effects.
- [ ] Function outputs are structured and safe to include in turn context.
- [ ] Save mutation still flows through `game_commit_turn`.
- [ ] Tests cover new or changed deterministic functions.

## Validation Gates

- `driver_script_runs_only_declared_starlark_functions`
- Targeted script tests in `crates/game`.
- TUI `game_run_driver` tests for tool-facing behavior when changed.
