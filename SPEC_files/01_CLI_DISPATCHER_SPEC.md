# CLI Dispatcher Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The CLI dispatcher owns the supported `deepseek` command. It routes install- and
terminal-facing flows into the companion TUI runtime while preserving stable
command behavior for users, scripts, installers, and package managers.

## Source Anchors

Primary code:

- `crates/cli/src/main.rs`
- `crates/cli/src/lib.rs`
- `crates/tui/src/main.rs`

Related code:

- `npm/`
- `Dockerfile`
- `config.example.toml`

Canonical docs:

- `README.md`
- `README.zh-CN.md`
- `docs/INSTALL.md`
- `docs/DOCKER.md`
- `docs/RELEASE_RUNBOOK.md`

Tests and fixtures:

- `crates/tui/tests/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/01_CLI_DISPATCHER_SPEC.md
Goal:
Command or install path affected:
Current behavior:
Desired behavior:
Compatibility requirements:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- `deepseek` is the canonical user command.
- The dispatcher delegates interactive and compatibility flows to
  `deepseek-tui`.
- `cargo run --bin deepseek` and `cargo run -p deepseek-tui-cli` are source-run
  paths.
- Release binaries and installers must keep the dispatcher and TUI companion in
  sync.

## Design Principles

- The user's mental model is one command: `deepseek`.
- Dispatcher behavior should be boring, stable, and scriptable.
- Compatibility aliases are maintained unless a breaking change is explicitly
  approved.
- Install docs must match actual binary behavior.

## Change Workflow

- Inspect both `crates/cli` and `crates/tui/src/main.rs` before changing CLI
  behavior.
- Check whether the command is user-facing, automation-facing, or internal.
- Update README, install docs, shell completions, and release notes when command
  behavior changes.
- For new subcommands, verify help text, error messages, and delegation path.

## Acceptance Criteria Checklist

- [ ] `deepseek <flow>` reaches the intended runtime path.
- [ ] Help output and docs match the behavior.
- [ ] Existing command aliases still work or are intentionally deprecated.
- [ ] Tests or manual command evidence cover the changed path.

## Validation Gates

- `cargo build`
- `cargo test -p deepseek-tui-cli`
- Relevant `cargo run --bin deepseek -- <args>` smoke tests.

Use the full workspace test gate for broad command routing changes.

## Risks

- Documenting `deepseek-tui` as the primary command splits the user model.
- Changing dispatcher behavior can break npm, Cargo, Homebrew, Docker, or
  direct-download installs.
- A TUI-only change can still require dispatcher updates if the entry point
  changes.
