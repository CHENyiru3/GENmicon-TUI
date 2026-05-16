# Galgame Driver Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The `galgame` driver is the minimal visual-novel style driver used to prove the
Game TUI loop: emotional dialogue, relationship scoring, plot/state helpers,
active NPC dialogue packs, and deterministic scoring.

## Source Anchors

Driver package:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/driver.toml`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/scripts/affection.star`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/agent_templates/`

Affected game:

- `SPEC_files/games/reconciliation-demo.md`
- `examples/games/reconciliation-demo/`

Framework prompt:

- `crates/tui/src/prompts/game_console.md`

Runtime code:

- `crates/game/src/driver.rs`
- `crates/game/src/script.rs`
- `crates/game/src/agents.rs`

## Maintainer Prompt

```markdown
Spec: SPEC_files/game_driver/drivers/galgame.md
Goal:
Affected game:
Current behavior:
Desired behavior:
Relationship/dialogue mechanics:
Driver function changes:
Agent role changes:
Acceptance criteria:
Validation I expect:
```

## Driver Contract

- Driver ID: `galgame`
- Current version: `0.1.0`
- Runtime: `starlark`
- Default topology: `dynamic-main-plus-managers`
- Entry skill: `skills/galgame/SKILL.md`
- Declared function: `score_action`
- Default roles: `state`, `plot`, `dialogue`
- Maximum active roles: `3`
- Templates: `state`, `plot`, `dialogue`

## Current Behavior

- `score_action` calculates deterministic action scoring through
  `scripts/affection.star`.
- Ordinary `score_action` results stay within the driver's declared scoring
  range. Cartridge-specific terminal sentinels, such as the reconciliation
  demo's `relationship_score = -100` violent failure marker, belong in that
  game's save contract and commit normalization rather than in reusable driver
  semantics.
- The generic `dialogue` role can expand into an active NPC pack, such as
  `dialogue_girlfriend`, based on save data.
- The driver supports the reconciliation demo's chat, move, and reflection
  action style without owning Rei-specific facts.
- The driver skill owns reusable galgame scoring and genre policy only. Shared
  language/dialogue restrictions, silent tool execution, player-mode hiding,
  fact-gate order, and commit orchestration belong to the Game Console prompt.

## Compatibility

- `reconciliation-demo` depends on `[driver] id = "galgame"` and version
  requirement `^0.1`.
- Saves should lock the resolved concrete driver version.
- A behavior change that alters scoring semantics should either remain
  compatible with `0.1.0` saves or require an intentional driver version bump.

## Acceptance Criteria Checklist

- [ ] `driver.toml` still declares only supported functions and role templates.
- [ ] `score_action` remains deterministic and side-effect free.
- [ ] Dynamic dialogue role expansion works for the active NPC.
- [ ] Driver skills do not duplicate Game Console controller or guardrail
      rules.
- [ ] `reconciliation-demo` docs/specs describe any changed behavior.
- [ ] Runtime tests cover changed scoring or role behavior.
