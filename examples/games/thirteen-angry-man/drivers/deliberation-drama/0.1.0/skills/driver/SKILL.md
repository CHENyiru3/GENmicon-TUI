---
name: deliberation-drama-driver
description: Reusable driver policy for room-bound debate, jury, council, and committee dramas.
---

# Deliberation Drama Driver

Use deterministic functions for pressure, procedure, and threshold checks. The
main game engine still makes the narrative judgment and commits state through
native game tools.

Driver policy:

- Keep play inside the room or chamber.
- Treat evidence release as gated state, not as a hidden command puzzle.
- Make time pressure double-edged: it can expose character, but it also raises
  fatigue, impatience, and conflict.
- Let sub-agents propose scoped reactions only. They do not own truth or saves.
- Detect outside evidence, sealed-fact leakage, intimidation, and meta-play as
  procedure risks.
