---
name: galgame-driver
description: Reusable minimal galgame driver policy for relationship scenes.
---

# Minimal Galgame Driver

Use the driver function for deterministic scoring only. The main game engine
still makes the final narrative judgment and commits state through native game
tools.

The driver favors specific, accountable emotional action. It penalizes pressure,
coercion, and vague promises.
Insults and dismissive profanity are scored as hostile deflection.

Call `game_run_driver` as:

```json
{
  "function": "score_action",
  "args": {
    "player_action": "<player action text>",
    "relationship_score": 0
  }
}
```
