# Sub-Agents And RLM Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns parallel and recursive assistance: sub-agent lifecycle tools,
TUI routing for child work, RLM sandboxed Python analysis, and long-session
delegation patterns that keep the parent session manageable.

## Source Anchors

Primary code:

- `crates/tui/src/tools/subagent/`
- `crates/tui/src/tui/subagent_routing.rs`
- `crates/tui/src/tools/rlm.rs`
- `crates/tui/src/rlm/`
- `crates/tui/src/repl/`

Related code:

- `crates/tui/src/tools/registry.rs`
- `crates/tui/src/core/engine/tool_setup.rs`
- `crates/tui/src/task_manager.rs`
- `crates/tui/src/prompts.rs`

Canonical docs:

- `docs/SUBAGENTS.md`
- `docs/TOOL_SURFACE.md`
- `PROMPT_ANALYSIS.md`

Tests and fixtures:

- `crates/tui/src/tools/subagent/tests.rs`
- Unit tests beside RLM and REPL modules

## Maintainer Prompt

```markdown
Spec: SPEC_files/07_SUBAGENTS_RLM_SPEC.md
Goal:
Sub-agent or RLM behavior affected:
Current behavior:
Desired behavior:
Routing/lifecycle expectations:
Safety constraints:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- The supported model-visible sub-agent surface is `agent_spawn` plus
  wait/result/cancel/list/send/resume/assign helpers.
- Removed swarm surfaces must not be reintroduced.
- RLM exposes a sandboxed Python REPL with in-REPL helpers such as
  `llm_query()` and `llm_query_batched()`.
- RLM helper names are not separate model-visible tools.
- Long sessions should delegate independent work and batch analysis to avoid
  parent-session bloat.

## Design Principles

- Sub-agents are for parallel, bounded work with clear ownership.
- Parent sessions should keep coordinating while children work.
- Child results must be inspectable, cancellable, and resumable.
- RLM is for contained analysis and classification, not arbitrary privilege
  escalation.

## Change Workflow

- Identify whether the change affects model-visible tools, UI routing, task
  lifecycle, prompt guidance, or RLM sandbox behavior.
- Update `docs/SUBAGENTS.md` and tool descriptions with code changes.
- Test lifecycle transitions: spawn, wait, result, send input, cancel, resume.
- For RLM changes, test sandbox behavior and helper availability.

## Acceptance Criteria Checklist

- [ ] Agent lifecycle actions are correct and observable in the TUI.
- [ ] Parent session is not blocked unnecessarily by child work.
- [ ] Removed swarm tools remain absent.
- [ ] RLM helpers work only in the intended REPL context.
- [ ] Docs explain the supported surface and limitations.

## Validation Gates

- Sub-agent unit tests.
- RLM/REPL unit tests.
- `cargo test -p deepseek-tui --all-features`.

## Risks

- Reintroducing broad swarm-style APIs can make session behavior hard to reason
  about.
- Poor child-result routing can lose work or duplicate edits.
- RLM sandbox changes can accidentally broaden execution authority.
