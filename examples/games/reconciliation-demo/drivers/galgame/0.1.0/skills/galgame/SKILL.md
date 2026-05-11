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

The Game Turn Controller owns language, narration, fact gates, branch
invariants, and commit authority. This driver skill is only the scoring policy.
If the playbook declares action skills, score the player's wording after it has
been routed through exactly one declared action skill. Do not create driver-only
actions outside that set. Reflection-style actions should produce a slight
nudge, not a score-optimizing answer.

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
