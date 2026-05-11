# Persistence And Recovery Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns session saves, checkpoints, state migrations, snapshot-based
workspace recovery, restore flows, schema compatibility, and durable state
integrity.

## Source Anchors

Primary code:

- `crates/tui/src/session_manager.rs`
- `crates/tui/src/schema_migration.rs`
- `crates/tui/src/snapshot/`
- `crates/tui/src/runtime_threads.rs`
- `crates/tui/src/tui/persistence_actor.rs`

Related code:

- `crates/state/src/lib.rs`
- `crates/tui/src/commands/session.rs`
- `crates/tui/src/commands/restore.rs`
- `crates/tui/src/tools/revert_turn.rs`
- `crates/tui/src/core/session.rs`
- `crates/tui/src/composer_history.rs`
- `crates/tui/src/composer_stash.rs`

Canonical docs:

- `docs/ARCHITECTURE.md`
- `docs/OPERATIONS_RUNBOOK.md`
- `README.md`

Tests and fixtures:

- Snapshot/session unit tests where present
- State crate tests
- `crates/tui/tests/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/10_PERSISTENCE_RECOVERY_SPEC.md
Goal:
State or recovery surface affected:
Current behavior:
Desired behavior:
Backward compatibility requirements:
Data migration requirements:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- Session manager handles saved sessions and checkpoints.
- Snapshot logic uses a side-git style mechanism for pre/post-turn workspace
  snapshots and `/restore`/`revert_turn` flows.
- Schema migration protects persisted state.
- Runtime threads store replayable events for headless/runtime workflows.

## Design Principles

- Persisted data must remain readable across upgrades unless a migration is
  explicitly provided.
- Recovery features must not touch the user's real `.git` history
  destructively.
- Restore flows should be explainable before they modify files.
- Long-session state must be managed to avoid unbounded crash-prone growth.

## Change Workflow

- Identify all persisted formats touched by the change.
- Add migrations or compatibility readers before changing serialized shape.
- Test old-to-new behavior with fixtures where practical.
- Update operations docs when recovery procedure changes.

## Acceptance Criteria Checklist

- [ ] Existing sessions or state files continue to load or migrate.
- [ ] Restore/revert behavior is bounded and user-visible.
- [ ] Snapshot pruning and storage growth are considered.
- [ ] Runtime thread replay remains consistent.
- [ ] Data-loss risks are documented and tested.

## Validation Gates

- Targeted session/snapshot/migration tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for `crates/state` changes.

## Risks

- Serialization changes without migration can strand user sessions.
- Restore bugs can overwrite user work.
- Session growth can degrade or crash long-running TUI sessions.
