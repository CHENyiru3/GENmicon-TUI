---
name: game-action-router
description: Parse player game input into declared action skills, bracket commands, choices, and custom wording without losing free-form intent.
---

# Game Action Router

Use this skill when a Game Console turn needs help interpreting player input.

Treat the active save's `interaction` block as the player-facing command menu:

- If `interaction.skills` / `playbook.action_skills` is present, every player
  action must be distilled to exactly one listed action skill. No other action
  category is valid.
- A bare number selects the matching `interaction.suggestions` entry.
- A bracket command such as `[ASK]`, `[INSPECT]`, `[VOTE]`, or `[APOLOGIZE]` sets the action class.
- Free-form text after the command is still important player intent inside the
  selected action skill.
- If the player types only natural language, infer the closest declared action
  skill and preserve the original text in `player_input`.
- Profanity, insults, threats, and hostile lines are still player actions. Do
  not stop play or scold out of character; route them to the nearest in-scene
  consequence such as pressure, deflection, intimidation, trust loss, or
  procedure risk.
- If the action asserts a new fact about bodies, identity, family, law,
  location, or backstory, route it through the game's fact gate before
  narration. A custom action can be free-form without being allowed to rewrite
  established continuity.

Do not reject creative actions only because their wording is not listed. The
declared action skills are the parser boundary: creativity is allowed inside a
skill, not outside the skill set. If no declared skill fits, ask for a valid
in-world action using one of the listed skills.

For each turn, produce one clear resolved action before narration:

```text
action_class: <command or inferred class>
action_skill: <declared skill id, when present>
target_node: <suggestion target or plausible story node>
player_intent: <short restatement>
```

Then use the normal game tools. State is authoritative only after `game_commit_turn`.

Reliability rules:

- If a numbered choice is out of range, treat the text as free-form intent when
  `freeform_allowed` is true; otherwise ask for a valid choice in character.
- If a bracket command is unknown, map it to the nearest declared verb and keep
  the raw command in `player_input`.
- If `action_skills` are present, do not invent a new verb, subsystem, or route;
  select one listed action skill or ask the player to rephrase.
- Never discard the player's exact wording. Use it for tone, target, and risk
  even when the action class is inferred.
- Do not rewrite recommended choices on every turn. Update them only when the
  story is drifting away from the active premise or the player needs a slight,
  non-spoiling reorientation.
- Do not let parser failure block the game loop. Fall back to the current
  active node and offer two or three concrete next actions.
- Do not present choices before the player knows who is in the scene and what
  was just said. If needed, call `game_playbook` and include the scene frame.
