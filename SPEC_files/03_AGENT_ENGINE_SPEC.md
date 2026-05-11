# Agent Engine Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The agent engine owns turn execution: session state, event flow, message
assembly, streaming, tool orchestration, capacity guardrails, coherence state,
post-tool hooks, and communication back to the TUI/runtime surfaces.

## Source Anchors

Primary code:

- `crates/tui/src/core/engine.rs`
- `crates/tui/src/core/engine/`
- `crates/tui/src/core/session.rs`
- `crates/tui/src/core/turn.rs`
- `crates/tui/src/core/events.rs`
- `crates/tui/src/core/ops.rs`

Related code:

- `crates/tui/src/core/capacity.rs`
- `crates/tui/src/core/capacity_memory.rs`
- `crates/tui/src/core/coherence.rs`
- `crates/tui/src/core/tool_parser.rs`
- `crates/tui/src/core/engine/lsp_hooks.rs`
- `crates/core/src/lib.rs`

Canonical docs:

- `docs/ARCHITECTURE.md`
- `docs/capacity_controller.md`
- `docs/TOOL_SURFACE.md`

Tests and fixtures:

- Unit tests beside engine modules
- `crates/tui/tests/`
- Mock LLM support in `crates/tui/src/llm_client/mock.rs`

## Maintainer Prompt

```markdown
Spec: SPEC_files/03_AGENT_ENGINE_SPEC.md
Goal:
Turn or event behavior affected:
Current behavior:
Desired behavior:
Tool/LLM/session implications:
Failure behavior:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- The engine is event-driven and coordinates model streaming, tool parsing,
  tool execution, approvals, LSP diagnostics, capacity checks, and transcript
  updates.
- `crates/tui` remains the active engine runtime even though extracted crates
  exist.
- DeepSeek thinking plus tool calls require reasoning-content replay in later
  requests.

## Design Principles

- Preserve turn determinism and replayability where practical.
- Keep model-visible tool setup explicit per mode and per context.
- Prefer mock LLM tests for engine behavior over HTTP mocking.
- Do not hide capacity or coherence interventions from the user.

## Change Workflow

- Map the change to turn setup, streaming, tool execution, event emission, or
  session mutation before editing.
- Check whether the same behavior exists in extracted `crates/core`.
- Add regression tests with mock LLM responses for engine behavior.
- For tool-related changes, update `SPEC_files/05_TOOL_SURFACE_SPEC.md` too.

## Acceptance Criteria Checklist

- [ ] Engine state transitions are explicit and tested.
- [ ] Tool calls, approvals, and errors produce correct events.
- [ ] Reasoning-content replay remains valid for DeepSeek thinking models.
- [ ] Capacity or compaction behavior is documented when changed.
- [ ] Resumed sessions behave consistently with fresh sessions.

## Validation Gates

- Targeted engine unit tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for cross-crate engine or protocol changes.

## Risks

- Small turn-loop changes can break streaming, tool calls, replay, or session
  persistence.
- Event ordering bugs may only show up under concurrent tools, approvals,
  sub-agents, or long output.
