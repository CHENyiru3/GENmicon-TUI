# Deliberation Drama Driver Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The `deliberation-drama` driver provides reusable mechanics for a fixed-room
jury deliberation game: vote pressure, procedure risk, room progression, NPC
management, and fair-process constraints.

## Source Anchors

Driver package:

- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/driver.toml`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/scripts/deliberation.star`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/agent_templates/`

Affected game:

- `SPEC_files/games/thirteen-angry-man.md`
- `examples/games/thirteen-angry-man/`

Runtime code:

- `crates/game/src/driver.rs`
- `crates/game/src/script.rs`
- `crates/game/src/agents.rs`

## Maintainer Prompt

```markdown
Spec: SPEC_files/game_driver/drivers/deliberation-drama.md
Goal:
Affected game:
Current behavior:
Desired behavior:
Vote/procedure/room mechanics:
Driver function changes:
Agent role changes:
Acceptance criteria:
Validation I expect:
```

## Driver Contract

- Driver ID: `deliberation-drama`
- Current version: `0.1.0`
- Runtime: `starlark`
- Default topology: `dynamic-main-plus-managers`
- Entry skill: `skills/driver/SKILL.md`
- Declared functions: `advance_room`, `evaluate_vote_change`,
  `detect_procedure_risk`
- Default roles: `state_manager`, `plot_manager`, `procedure_manager`,
  `npc_manager_a`, `npc_manager_b`
- Maximum active roles: `5`
- Templates: `state_manager`, `plot_manager`, `procedure_manager`,
  `npc_manager`

## Current Behavior

- `advance_room` calculates room progression.
- `evaluate_vote_change` evaluates vote pressure and likely movement.
- `detect_procedure_risk` identifies fair-process risks.
- NPC manager roles are bounded by driver topology while game content supplies
  specific juror facts.

## Compatibility

- `thirteen-angry-man` depends on `[driver] id = "deliberation-drama"` and
  version requirement `^0.1`.
- Save reload should use the concrete recorded driver version.
- New mechanics that change room/vote semantics should either be compatible
  with `0.1.0` saves or use a version bump and explicit migration decision.

## Acceptance Criteria Checklist

- [ ] `driver.toml` declares only supported deliberation functions.
- [ ] Functions remain deterministic and side-effect free.
- [ ] Role bounds match the game's `AGENTS.json` expectations.
- [ ] `thirteen-angry-man` specs/docs describe any changed behavior.
- [ ] Runtime tests cover changed vote, risk, room, or role behavior.
