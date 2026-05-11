# Tool Surface Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The tool surface owns model-visible built-in tools, tool schemas, registry
composition, mode-specific exposure, argument repair, result truncation,
approval integration, and compatibility aliases.

## Source Anchors

Primary code:

- `crates/tui/src/tools/`
- `crates/tui/src/tools/registry.rs`
- `crates/tui/src/tools/spec.rs`
- `crates/tui/src/core/engine/tool_setup.rs`

Related code:

- `crates/tools/src/lib.rs`
- `crates/tui/src/tools/arg_repair.rs`
- `crates/tui/src/tools/schema_sanitize.rs`
- `crates/tui/src/tools/truncate.rs`
- `crates/tui/src/tools/tool_result_retrieval.rs`
- `crates/tui/src/tools/large_output_router.rs`
- `crates/tui/src/tools/approval_cache.rs`

Canonical docs:

- `docs/TOOL_SURFACE.md`
- `docs/MODES.md`
- `docs/SUBAGENTS.md`
- `docs/MCP.md`

Tests and fixtures:

- Unit tests beside tool modules
- `crates/tui/src/tools/subagent/tests.rs`
- Tool integration tests under `crates/tui/tests/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/05_TOOL_SURFACE_SPEC.md
Goal:
Tool name or family:
Current behavior:
Desired behavior:
Mode exposure:
Approval/sandbox behavior:
Compatibility requirements:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- Built-in tools include shell, file operations, patching, git, search, web,
  GitHub, tasks, automation, plan/todo helpers, RLM, sub-agents, game tools,
  validation, and result retrieval.
- Registry composition and per-mode exposure determine what the model can see.
- Tool names are compatibility-sensitive and should remain stable.
- Large output and truncation helpers keep tool results usable inside long
  sessions.

## Design Principles

- Tool schemas are contracts. Rename only with explicit compatibility handling.
- Mode-specific exposure must be intentional and documented.
- Tool errors should be actionable for the model and safe for the user.
- Avoid broadening file, network, shell, or external-service capability without
  policy review.

## Change Workflow

- Identify whether the change is a new tool, schema change, result change,
  exposure change, or implementation change.
- Update prompts/tool docs/tests with code.
- Check approval and sandbox implications before exposing the tool in Agent,
  YOLO, Plan, game, or sub-agent contexts.
- Add compatibility aliases if replacing a model-visible name.

## Acceptance Criteria Checklist

- [ ] Tool schema matches implementation and docs.
- [ ] Mode exposure is correct and tested.
- [ ] Errors, truncation, and large-output retrieval behave predictably.
- [ ] Approval and sandbox policy are preserved or intentionally changed.
- [ ] Prompt docs and model-visible descriptions are updated.

## Validation Gates

- Targeted tool unit tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for shared `crates/tools` changes.

## Risks

- Small schema changes can break saved prompts, compatibility aliases, or model
  tool selection.
- Exposing a tool in the wrong mode can bypass the user's expected safety model.
- Web, shell, GitHub, and MCP surfaces need extra review for trust boundaries.
