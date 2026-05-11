# Approval, Sandbox, And Modes Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns the safety model for Plan, Agent, and YOLO modes; command
approval; persistent approval rules; sandbox policy; and execution decisions for
tools that can affect files, processes, network, or external systems.

## Source Anchors

Primary code:

- `crates/tui/src/tui/app.rs`
- `crates/tui/src/core/engine/tool_setup.rs`
- `crates/tui/src/core/engine/approval.rs`
- `crates/tui/src/tui/approval.rs`
- `crates/tui/src/execpolicy/`
- `crates/tui/src/sandbox/`

Related code:

- `crates/execpolicy/src/lib.rs`
- `crates/tui/src/tools/shell.rs`
- `crates/tui/src/tools/apply_patch.rs`
- `crates/tui/src/workspace_trust.rs`
- `crates/tui/src/command_safety.rs`
- `crates/tui/src/network_policy.rs`

Canonical docs:

- `docs/MODES.md`
- `docs/TOOL_SURFACE.md`
- `docs/ARCHITECTURE.md`

Tests and fixtures:

- Unit tests under execpolicy and sandbox modules
- TUI approval tests where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/06_APPROVAL_SANDBOX_MODES_SPEC.md
Goal:
Mode or policy affected:
Current behavior:
Desired behavior:
Commands/tools affected:
User approval expectations:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- Plan mode is read-only investigation.
- Agent mode uses approval gates for sensitive tool actions.
- YOLO mode auto-approves within configured policy.
- Sandbox backends include platform-specific preparation and denial reporting.
- Exec policy decides whether commands are allowed, denied, or require approval.

## Design Principles

- User intent controls authority. Convenience must not silently broaden power.
- Approval prompts should be specific enough for a non-expert maintainer to
  decide.
- Persistent approval rules must be narrow and explainable.
- Destructive operations require explicit approval unless the user directly
  requested them.

## Change Workflow

- Classify the affected capability: file write, process execution, network,
  git, external API, credentials, or UI-only.
- Check all modes and sub-agent contexts for exposure.
- Update docs and prompts when policy behavior changes.
- Add tests for allow/deny/approval edge cases.

## Acceptance Criteria Checklist

- [ ] Plan mode cannot perform unintended writes or side effects.
- [ ] Agent mode requests approval with clear command/action context.
- [ ] YOLO behavior remains bounded by configured policy.
- [ ] Sandbox denial messages are useful and safe.
- [ ] Persistent approval prefix behavior is tested.

## Validation Gates

- Targeted execpolicy tests.
- Targeted sandbox tests for the affected platform logic.
- `cargo test -p deepseek-tui --all-features`.

## Risks

- Approval rule prefixes that are too broad can authorize future unsafe
  commands.
- Sandbox behavior differs across macOS, Linux, and Windows helper paths.
- Tool exposure changes can accidentally weaken Plan mode.
