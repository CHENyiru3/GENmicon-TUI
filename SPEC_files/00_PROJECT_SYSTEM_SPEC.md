# Project System Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec defines the project-wide management layer for DeepSeek TUI. It keeps
feature work traceable from maintainer prompt to code, tests, docs, and release
evidence.

## Source Anchors

Primary project files:

- `AGENTS.md`
- `Cargo.toml`
- `README.md`
- `README.zh-CN.md`
- `CHANGELOG.md`

Canonical docs:

- `docs/ARCHITECTURE.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/RELEASE_RUNBOOK.md`
- `docs/OPERATIONS_RUNBOOK.md`

Spec files:

- `SPEC_files/README.md`
- `SPEC_files/WORKFLOW.md`
- `SPEC_files/SPEC_TEMPLATE.md`

## Maintainer Prompt

```markdown
Spec: SPEC_files/00_PROJECT_SYSTEM_SPEC.md
Goal:
Why it matters:
Affected project standards:
Must include:
Must not include:
Acceptance criteria:
Validation I expect:
```

## Project-Wide Rules

- The canonical command is `deepseek`; do not document `deepseek-tui` as the
  primary user entry point.
- The workspace must compile on stable Rust 1.88 or newer.
- Do not introduce nightly-only features, `#![feature(...)]`, or `cargo
  +nightly` requirements.
- Treat `crates/tui` as the live end-user runtime unless the code has already
  been fully moved into an extracted crate.
- Use the repo's existing patterns before creating new abstractions.
- Keep tool names, command names, config keys, and compatibility aliases stable
  unless the spec explicitly approves a breaking change.
- Update code, tests, help/localization, and docs together for shipped
  behavior.

## Definition Of Done

A change is complete only when:

- The maintainer goal is restated as concrete deliverables.
- Every explicit requirement has evidence.
- User-facing behavior is implemented and documented.
- Regression tests or appropriate coverage exist for risky behavior.
- Validation commands have been run, or the reason they could not run is stated.
- Relevant spec files are updated if behavior or ownership changed.
- Unrelated worktree changes are not reverted or mixed into the change.

## Standard Validation Ladder

Use the narrowest useful checks while developing, then broaden before handoff:

- Formatting: `cargo fmt --all --check`
- Build: `cargo build`
- Tests: `cargo test --workspace --all-features`
- Lint: `cargo clippy --workspace --all-targets --all-features`

Docs-only changes usually do not require the full Rust gate, but they still need
file-level review and link/path checks.

## Traceability Checklist

For each feature or fix:

- Prompt and spec file identified.
- Code anchors inspected.
- Acceptance criteria written.
- Implementation files listed.
- Tests and docs listed.
- Validation output summarized.
- Remaining risks recorded.

## Risks

- Specs can drift from code if they are not updated in the same PR as behavior.
- Broad specs can hide ownership. Split a new spec when a surface becomes a
  recurring workstream.
- Passing tests can be a weak signal if no test covers the maintainer's actual
  requirement.
