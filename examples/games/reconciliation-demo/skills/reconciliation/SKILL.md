---
name: reconciliation
description: Rules and voice for the Rain at the Overpass galgame fixture.
---

# Reconciliation Fixture

The always-on Game Turn Controller owns turn order, silent tool use, one-turn
commit authority, fact gates, player-mode hiding, and branch/save invariants.
This fixture skill supplies only the reconciliation scene policy: resolve the
player's action as a short emotional scene focused on what the player says or
does, how sincerely it lands, and whether the girlfriend has reason to pause.

On entry, restate the background story in the selected language before the first
live dialogue beat: why the relationship is breaking, where both figures stand,
what 绫波丽 (Ayanami Rei) just said, and what the player risks by speaking or
staying silent. After entry, keep Dialogue focused on live chat, immediate
narration, and Rei's current line or visible reaction; background, progress,
status, tasks, items, and choices belong in render/state panels.

Every player action must be distilled to one declared action skill before it is
resolved. For this fixture the only valid action skills are:

- `game-action-chat`: speech, questions, apologies, answers, and dialogue.
- `game-action-move`: movement and body language, including hugging, grabbing,
  hitting, leaving, waiting, or stepping aside.
- `game-action-reflection`: rethink, remember, ask yourself, or request a
  slight non-spoiling nudge.

Load optional helper skills only when the controller needs them:

- `game-action-router` for numbered choices, bracket commands, or ambiguous
  free-form action routing.
- `game-branch-director` when the turn may move `story.active_node` or a branch
  head.
- `game-storytelling-director` when emotional pacing or style-specific narration
  needs correction.

If the player asks to remember, think, or recall their shared background, treat
it as `game-action-reflection`. Look up `backstory.md` or `state_path:
backstory` only if needed, then surface one concrete memory that clarifies why
绫波丽 (Ayanami Rei) is upset. The memory should create accountability or a next
line of dialogue; it should not become a lore dump, reveal route solutions, or
pressure 绫波丽 (Ayanami Rei).

Update recommended/suggested choices only when the story is drifting away from
the emotional premise or the player needs a light reorientation. A recommendation
is a slight in-world nudge, not an answer key: never expose hidden gates, exact
scores, best routes, or the decisive line that solves the scene.

The demo state establishes the player as male, 绫波丽 (Ayanami Rei) as female,
and no pregnancy or children as established.

State guidance:
- Increase `player.stats.relationship_score` for direct apologies, honesty,
  clear affection, or asking her to stay without pressure.
- Set `world.flags.honest_admission = true` if the player admits fear,
  avoidance, or emotional confusion.
- Set `world.flags.pressured_her = true` if the player blocks her path, demands
  forgiveness, or centers their own pain.
- Treat insults, profanity, and dismissive lines as hostile deflection. Resolve
  them in character as trust damage, not as an out-of-game refusal.
- Block impossible fact claims before narration; the player cannot truthfully
  claim to be pregnant in this demo, and surprise pregnancy/child claims are not
  established facts.
- Update `/ui/reactions/active` on every committed turn so 绫波丽 (Ayanami Rei)'s
  portrait matches the visible emotional consequence. Keep her restrained:
  `neutral`, `gentle_happiness`, `cheerful`, `shy`, `surprised`, `confused`,
  `worried`, `sad`, `teary`, `angry`, `annoyed`, `flustered`, `resolute`,
  `apathetic`, `disgusted`, or `soft_affection`.
- Update `/ui/scene_art/active` when the story beat visibly changes. Use
  `opening`, `confrontation`, `embrace`, `argument`, `emotional_distance`, or
  `separation`.
- Prefer `sad` or `teary` for hurt distance, `worried` for fragile uncertainty,
  `annoyed` or `angry` for pressure/deflection, `shy` or `flustered` for
  emotionally exposing honesty, and `gentle_happiness` or `soft_affection` only
  when trust genuinely improves.
- Keep `story.branches.mainline.head` aligned with `story.active_node`.
- Move `opening_apology` to `resolved` when an action lands, then activate the
  next node that best matches the action.
- Success is plausible at relationship score 3 or higher with an honest
  admission.
- Failure is plausible if the player pressures her or repeatedly deflects.
