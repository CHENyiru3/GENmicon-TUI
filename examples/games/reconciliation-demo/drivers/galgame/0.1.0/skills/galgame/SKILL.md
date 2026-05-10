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

If the playbook declares action skills, route the player's wording through one
declared action skill before scoring. Do not create driver-only actions outside
that set. Reflection-style actions should produce a slight nudge, not a score-
optimizing answer.

The play language is selected before the TUI session starts. Do not ask for it
inside the story. Use only English or Chinese, and keep all player-facing text
aligned to the selected language. The opening must always restate the
background/reason she is upset before the first live dialogue beat. Keep
background and progress in render/state panels; keep dialogue output focused on
chat and immediate emotional consequence. The Dialogue pane is plain text, so do
not use Markdown-only headings, rules, bold markers, or raw Markdown lists
there.

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
