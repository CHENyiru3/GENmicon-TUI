# <Game Title> Game Spec

Status: Draft
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

Describe this single game cartridge and what kind of experience it should
deliver.

## Source Anchors

Game package:

- `examples/games/<game-id>/game.toml`
- `examples/games/<game-id>/GAME.md`
- `examples/games/<game-id>/content/`
- `examples/games/<game-id>/skills/`
- `examples/games/<game-id>/saves/`

Driver:

- `SPEC_files/game_driver/drivers/<driver-id>.md`

## Maintainer Prompt

```markdown
Spec: SPEC_files/games/<game-id>.md
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

- Game ID:
- Title:
- Version:
- Entry skill:
- Default save:
- Driver:
- Supported languages:
- Player role:
- Core loop:

## Content Contract

- Fixed facts:
- Mutable state:
- Allowed player actions:
- Invalid player actions:
- Endings:
- Fact-check policy:

## Save Contract

- Authoritative save files:
- Required state sections:
- Turn log expectations:
- Agent roster expectations:
- Restart behavior:

## Acceptance Criteria Checklist

- [ ] `game.toml` matches this spec.
- [ ] `GAME.md` explains current play behavior.
- [ ] Content and skills match the game contract.
- [ ] Save fixtures load and restart.
- [ ] Driver dependency is documented and compatible.
