# Driver Agent Topology Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns driver-declared game sub-agent roles, role templates, dynamic role
expansion, and scoped agent packs.

## Source Anchors

Primary code:

- `crates/game/src/agents.rs`
- `crates/game/src/driver.rs`
- `crates/tui/src/tools/subagent/`
- `crates/tui/src/tui/subagent_routing.rs`

Example templates:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/agent_templates/`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/agent_templates/`
- `examples/games/reconciliation-demo/saves/default/AGENTS.json`
- `examples/games/thirteen-angry-man/saves/default/AGENTS.json`

## Current Behavior

- Drivers declare default roles and a maximum active role count.
- Drivers map reusable role names to agent template files.
- The galgame driver can expand a generic `dialogue` role into an active-NPC
  pack such as `dialogue_girlfriend`.
- Game-scoped `game_agent_*` helpers restrict child agents to declared packs.

## Design Principles

- Driver topology sets bounds; game save state selects active packs.
- Child agents propose; the main game session commits.
- Templates must stay under the driver root.
- Player mode should not expose normal coding sub-agent power.

## Acceptance Criteria Checklist

- [ ] Active packs do not exceed driver `max_active`.
- [ ] Role names and templates are validated.
- [ ] Dynamic roles stay within driver-declared role families.
- [ ] Child packs include only scoped game/save/driver context.
- [ ] Tests cover role bounds and dynamic role expansion.

## Validation Gates

- `agent_packs_are_limited_to_driver_declared_roles`
- Dynamic dialogue role tests in `crates/game`.
- Sub-agent routing tests when TUI helper behavior changes.
