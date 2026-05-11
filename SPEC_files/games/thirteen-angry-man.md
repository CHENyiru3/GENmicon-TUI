# Thirteen Angry Man Game Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

`thirteen-angry-man` is the first serious-game cartridge scaffold for Game TUI.
It is a fixed-room deliberation drama where the player is Juror 13 and the
central play is fair process under pressure, not investigation.

## Source Anchors

Game package:

- `examples/games/thirteen-angry-man/game.toml`
- `examples/games/thirteen-angry-man/GAME.md`
- `examples/games/thirteen-angry-man/content/`
- `examples/games/thirteen-angry-man/skills/deliberation/SKILL.md`
- `examples/games/thirteen-angry-man/saves/`

Driver:

- `SPEC_files/game_driver/drivers/deliberation-drama.md`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/`

Runtime tests:

- `crates/game/src/tests.rs`

## Maintainer Prompt

```markdown
Spec: SPEC_files/games/thirteen-angry-man.md
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

- Game ID: `thirteen-angry-man`
- Title: `Thirteen Angry Man`
- Version: `0.1.0`
- Entry skill: `deliberation`
- Default save: `default`
- Driver: `deliberation-drama` with requirement `^0.1`
- Player role: Juror 13.
- Core loop: question, slow the room, request votes, inspect admitted evidence
  summaries, and protect fair process.

## Content Contract

- Fixed case facts live under `content/`.
- Runtime truth lives in `saves/default/STATE.json` and
  `saves/default/TURN_LOG.jsonl`.
- The game is not an investigation game.
- The player cannot leave the room, call witnesses, or introduce new evidence.
- Admitted evidence can be inspected through content-backed summaries.
- Play should test doubt, fatigue, pride, prejudice, civic pressure, and room
  procedure.

## Save Contract

- Authoritative save files include `STATE.json`, `TURN_LOG.jsonl`,
  `SUMMARY.md`, and `AGENTS.json`.
- `story.active_branch` and `story.branches.<name>.head` identify the current
  route.
- Each committed turn is appended to `TURN_LOG.jsonl`.
- Normal play must not write to repository git history.

## Driver Dependency

This game depends on the `deliberation-drama` driver. Game-specific changes
should not edit reusable vote, risk, room, or role mechanics unless
`SPEC_files/game_driver/drivers/deliberation-drama.md` is updated in the same
change.

## Acceptance Criteria Checklist

- [ ] `game.toml` still matches this spec.
- [ ] `GAME.md` explains current play commands and deliberation limits.
- [ ] Content updates do not turn the game into open investigation unless the
      spec explicitly changes.
- [ ] Save fixtures remain loadable and restartable.
- [ ] Driver role bounds match the save's agent roster.
- [ ] Tests or manual evidence cover load, choices/render, commit, and restart
      paths affected by the change.
