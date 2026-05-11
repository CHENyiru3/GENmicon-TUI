# Rain At The Overpass Game Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

`reconciliation-demo` is the minimal Game TUI fixture for a galgame-style
reconciliation scene. It proves load, render, choices, bounded lookup,
deterministic driver calls, JSON Merge Patch commits, agent reconstruction, and
restart from authoritative saves.

## Source Anchors

Game package:

- `examples/games/reconciliation-demo/game.toml`
- `examples/games/reconciliation-demo/GAME.md`
- `examples/games/reconciliation-demo/content/`
- `examples/games/reconciliation-demo/skills/reconciliation/SKILL.md`
- `examples/games/reconciliation-demo/saves/`
- `examples/games/reconciliation-demo/save_templates/`

Driver:

- `SPEC_files/game_driver/drivers/galgame.md`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/`

Framework prompt:

- `crates/tui/src/prompts/game_console.md`

Runtime tests:

- `crates/game/src/tests.rs`

## Maintainer Prompt

```markdown
Spec: SPEC_files/games/reconciliation-demo.md
Goal:
Player-facing change:
Story/mechanics affected:
Current behavior:
Desired behavior:
Files likely affected:
Driver changes needed:
Acceptance criteria:
Validation I expect:
```

## Game Contract

- Game ID: `reconciliation-demo`
- Title: `Rain at the Overpass`
- Version: `0.1.0`
- Entry skill: `reconciliation`
- Default save: `default`
- Driver: `galgame` with requirement `^0.1`
- Player role: partner trying to repair trust with Ayanami Rei at a station
  overpass during evening rain.
- Core loop: player chooses or writes a chat, movement, or reflection action;
  the game resolves emotional consequence, validates facts, updates state, and
  commits a turn.

## Content Contract

- Fixed content lives under `content/`.
- `content/backstory.md` owns the full relationship background and why past
  memories only help when turned into accountability.
- Allowed action skills are `game-action-chat`, `game-action-move`, and
  `game-action-reflection`.
- Flexible wording is allowed inside those action skills.
- New biology, identity, family, legal, location, or backstory facts must pass
  `game_fact_check` before entering narration or committed state.
- Shared turn-control rules live in the Game Console prompt at
  `crates/tui/src/prompts/game_console.md`: silent tool execution,
  player-mode hiding, fact-gate order, save invariant protection, and
  commit-before-render discipline.
- The `reconciliation` entry skill should stay fixture-specific: scene voice,
  available action skills, state guidance, portrait/background guidance, and
  game-specific fact policy. It should not duplicate generic controller or
  guardrail text.
- The game should stay small enough to function as a framework proof fixture.

## Save Contract

- Authoritative live saves live under `saves/`.
- Save templates live under `save_templates/`.
- Required save files include `STATE.json`, `TURN_LOG.jsonl`, and
  `AGENTS.json`.
- `story.branches.mainline.head` points at the active beat.
- `TURN_LOG.jsonl` records committed turns.
- `AGENTS.json` declares restartable processors: `state`, `plot`, and
  `dialogue_girlfriend`.
- `dialogue_girlfriend` is the active-NPC expansion of the driver-level
  `dialogue` role.

## Driver Dependency

This game depends on the `galgame` driver. Game-specific changes should not edit
driver scoring, reusable role templates, or driver manifest behavior unless
`SPEC_files/game_driver/drivers/galgame.md` is updated in the same change.

## Acceptance Criteria Checklist

- [ ] `game.toml` still matches this spec.
- [ ] `GAME.md` explains current play commands and action skills.
- [ ] Content updates do not introduce unsupported facts without fact-check
      policy updates.
- [ ] Entry, action, NPC, and driver skills do not duplicate Game Console
      controller rules.
- [ ] Save templates and live saves remain restartable.
- [ ] Driver role expansion still produces `dialogue_girlfriend`.
- [ ] Tests or manual evidence cover load, choices/render, commit, and restart
      paths affected by the change.
