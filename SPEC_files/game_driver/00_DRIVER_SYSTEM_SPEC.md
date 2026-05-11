# Game Driver System Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The game driver system owns reusable driver packages for Game TUI. It separates
genre mechanics and deterministic driver behavior from individual game content.

## Source Anchors

Primary code:

- `crates/game/src/driver.rs`
- `crates/game/src/script.rs`
- `crates/game/src/agents.rs`
- `crates/game/src/manifest.rs`
- `crates/game/src/save.rs`

Related code:

- `crates/tui/src/tools/game.rs`
- `crates/tui/src/core/engine/tool_setup.rs`
- `crates/tui/src/tools/registry.rs`

Canonical docs:

- `docs/GAME_TUI_FRAMEWORK_SPEC.md`
- `SPEC_files/13_GAME_TUI_FRAMEWORK_SPEC.md`
- `SPEC_files/game_driver/README.md`

## Current Behavior

- Game manifests name a driver ID and semver requirement.
- `DriverResolver` searches driver roots for matching installed versions.
- `load_driver` validates `driver.toml`, script paths, skill paths, and
  sub-agent templates under the driver root.
- `game_run_driver` can run declared driver functions.
- Save state records the concrete driver ID and version.

## Design Principles

- Drivers are reusable and versioned.
- Driver packages are untrusted filesystem input until validated.
- `crates/game` owns validation and deterministic runtime data.
- TUI owns presentation, LLM orchestration, and process adapters.
- Driver functions cannot be arbitrary file access or save writers.

## Change Workflow

- Update this shared spec for runtime-wide driver contract changes.
- Update a concrete `drivers/<driver-id>.md` spec for driver-specific changes.
- Update every affected game spec under `SPEC_files/games/`.
- Add tests in `crates/game` for manifest, version, script, and role behavior.

## Acceptance Criteria Checklist

- [ ] Driver root traversal is rejected.
- [ ] Driver ID and version are filesystem-safe and semver-valid.
- [ ] Driver functions are declared before they can run.
- [ ] Save reload enforces the exact recorded driver version where required.
- [ ] Driver behavior remains separate from single-game content.

## Validation Gates

- Targeted `crates/game` tests.
- `cargo test -p deepseek-game --all-features` when package invocation is
  available.
- Full workspace tests for TUI tool-profile or command changes.
