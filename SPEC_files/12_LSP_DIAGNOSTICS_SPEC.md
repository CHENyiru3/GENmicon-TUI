# LSP Diagnostics Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns language server integration and post-edit diagnostics. It makes
compiler/linter feedback available after file edits so the agent can repair
problems before the next turn.

## Source Anchors

Primary code:

- `crates/tui/src/lsp/mod.rs`
- `crates/tui/src/lsp/client.rs`
- `crates/tui/src/lsp/diagnostics.rs`
- `crates/tui/src/lsp/registry.rs`
- `crates/tui/src/core/engine/lsp_hooks.rs`

Related code:

- `crates/tui/src/tools/diagnostics.rs`
- `crates/tui/src/tools/file.rs`
- `crates/tui/src/tools/apply_patch.rs`
- `crates/tui/src/tools/spec.rs`

Canonical docs:

- `docs/ARCHITECTURE.md`
- `docs/TOOL_SURFACE.md`

Tests and fixtures:

- LSP unit tests where present
- Tool integration tests for post-edit diagnostics where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/12_LSP_DIAGNOSTICS_SPEC.md
Goal:
Language or diagnostic behavior affected:
Current behavior:
Desired behavior:
Server availability assumptions:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- LSP manager lazily starts per-language stdio clients.
- Language detection maps files to default servers such as rust-analyzer,
  pyright, gopls, clangd, and typescript-language-server.
- Diagnostics are injected after successful edit operations.
- Diagnostic output is rendered in a model-readable block.

## Design Principles

- Diagnostics should help repair code without making edits dependent on every
  language server being installed.
- Missing servers should degrade gracefully.
- Post-edit hooks must not block the turn indefinitely.
- Diagnostic rendering should be concise and actionable.

## Change Workflow

- Identify whether the change affects language detection, transport, lifecycle,
  rendering, or engine hook timing.
- Add tests for parser/render behavior and missing-server paths.
- Update docs when adding a supported language or changing default servers.
- Verify the edited tool path still triggers diagnostics.

## Acceptance Criteria Checklist

- [ ] Diagnostics run after relevant edit operations.
- [ ] Missing or failed language servers produce safe, useful behavior.
- [ ] New language mappings are documented and tested.
- [ ] Diagnostic rendering remains compact and parseable.
- [ ] Engine turn flow continues when diagnostics fail.

## Validation Gates

- Targeted LSP tests.
- `cargo test -p deepseek-tui --all-features`.
- Manual smoke test only when the relevant language server is installed.

## Risks

- Blocking LSP calls can slow or stall turns.
- Overly verbose diagnostics can bloat model context.
- Server lifecycle bugs can leak child processes.
