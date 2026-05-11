# MCP, Skills, Hooks, And Memory Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns extension systems: MCP clients/server mode, skills discovery and
activation, lifecycle hooks, and persistent user memory injected into agent
context.

## Source Anchors

Primary code:

- `crates/tui/src/mcp.rs`
- `crates/tui/src/mcp_server.rs`
- `crates/tui/src/skills/`
- `crates/tui/src/skill_state.rs`
- `crates/tui/src/hooks.rs`
- `crates/tui/src/memory.rs`

Related code:

- `crates/mcp/src/lib.rs`
- `crates/hooks/src/lib.rs`
- `crates/tui/src/tools/skill.rs`
- `crates/tui/src/tools/remember.rs`
- `crates/tui/src/commands/mcp.rs`
- `crates/tui/src/commands/skills.rs`
- `crates/tui/src/commands/hooks.rs`
- `crates/tui/src/commands/memory.rs`

Canonical docs:

- `docs/MCP.md`
- `docs/MEMORY.md`
- `docs/TOOL_SURFACE.md`
- `docs/CONFIGURATION.md`

Tests and fixtures:

- MCP, skills, hooks, and memory unit tests where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/11_MCP_SKILLS_HOOKS_MEMORY_SPEC.md
Goal:
Extension surface affected:
Current behavior:
Desired behavior:
Trust/safety requirements:
Config or command changes:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- MCP support manages external tool servers and server mode.
- Skills are composable instruction packs discovered and activated by the TUI.
- Hooks run lifecycle actions around tool events.
- Memory stores persistent user preferences for future context injection.

## Design Principles

- Extension input is untrusted until explicitly approved.
- External services, hosted endpoints, branding, telemetry, and credentials
  require maintainer approval before shipping.
- Memory should be useful, inspectable, and not a hidden policy override.
- Hooks must be predictable and auditable.

## Change Workflow

- Classify the change as MCP protocol, skill loading, hook execution, memory
  persistence, command UI, or config.
- Review trust boundaries and approval requirements.
- Update docs and examples for user-visible extension behavior.
- Add tests for failure modes and disabled/unavailable extensions.

## Acceptance Criteria Checklist

- [ ] Extension behavior is explicit and documented.
- [ ] External input cannot override project instructions or safety policy.
- [ ] Memory and hooks are inspectable and manageable by the user.
- [ ] Config/help text matches shipped behavior.
- [ ] Tests cover enabled, disabled, and failure paths.

## Validation Gates

- Targeted MCP/skills/hooks/memory tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for extracted MCP/hooks crate changes.

## Risks

- Prompt injection can arrive through MCP tools, skills, memory, or external
  docs.
- Hooks can create invisible side effects if not surfaced clearly.
- Memory can become stale or overbroad without user control.
