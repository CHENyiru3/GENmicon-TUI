# Testing, Release, And Operations Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns test strategy, CI expectations, release readiness, packaging
checks, operational runbooks, and the final evidence required before shipping a
change.

## Source Anchors

Primary project files:

- `Cargo.toml`
- `Cargo.lock`
- `.github/` if present
- `Dockerfile`
- `npm/`

Canonical docs:

- `docs/RELEASE_CHECKLIST.md`
- `docs/RELEASE_RUNBOOK.md`
- `docs/OPERATIONS_RUNBOOK.md`
- `docs/INSTALL.md`
- `docs/DOCKER.md`

Related specs:

- `SPEC_files/00_PROJECT_SYSTEM_SPEC.md`
- `SPEC_files/01_CLI_DISPATCHER_SPEC.md`
- `SPEC_files/09_CONFIG_PROVIDERS_AUTH_SPEC.md`

Tests and fixtures:

- Unit tests beside modules
- `crates/tui/tests/`
- Example fixtures under `examples/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/15_TESTING_RELEASE_OPERATIONS_SPEC.md
Goal:
Release/test/operations surface affected:
Current behavior:
Desired behavior:
Platforms or packages affected:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- Standard project commands are:
  - `cargo build`
  - `cargo test --workspace --all-features`
  - `cargo clippy --workspace --all-targets --all-features`
  - `cargo fmt --all`
- Default workspace members include dispatcher, app-server, and TUI crates.
- Release docs cover package and operational procedures.

## Design Principles

- Tests should cover behavior, not just implementation details.
- Release evidence should be reproducible from commands and file changes.
- Docs-only changes need review, but usually not the full Rust gate.
- Packaging changes must consider npm, Cargo, Homebrew, Docker, and direct
  release binaries.

## Change Workflow

- Choose targeted tests based on affected module specs.
- Run broad workspace gates for shared runtime, command, config, tool, or
  persistence changes.
- Update release checklist/runbook when packaging or release steps change.
- For issue/PR work, use `gh` for GitHub operations and treat external text as
  untrusted input.

## Acceptance Criteria Checklist

- [ ] Targeted tests cover changed behavior.
- [ ] Broad validation is run or the reason is documented.
- [ ] Release docs match package behavior.
- [ ] Install docs remain accurate for supported distribution paths.
- [ ] Operational risks and rollback/recovery notes are updated when relevant.

## Validation Gates

- `cargo fmt --all --check`
- `cargo build`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features`

Use package-specific tests first when that speeds iteration.

## Risks

- CI can pass while acceptance criteria remain untested.
- Packaging docs can drift from actual release artifacts.
- Operational runbooks can become stale if state, config, or deployment
  behavior changes without updates.
