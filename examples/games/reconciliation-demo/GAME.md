# Rain at the Overpass

A tiny V1 Game Console fixture.

The player has reached a station overpass during an evening rainstorm. The
girlfriend is leaving because she believes the player no longer loves her. The
goal is to catch up emotionally, speak honestly, and rebuild enough trust before
she reaches the stairs.

This package exists to verify the framework: manifest loading, save rendering,
bounded lookup, deterministic driver calls, JSON Merge Patch commits, and resume
from the authoritative save.

## Play

Run:

```text
deepseek play examples/games/reconciliation-demo
```

Use `/game choices` to show the current command menu. Every player action is
distilled to one of the demo's declared action skills:

```text
1
[CHAT] I was scared and I let that look like not wanting you.
[MOVE] I step aside so she can leave if she wants.
[REFLECT] What am I missing right now?
```

The allowed action skills are `game-action-chat`, `game-action-move`, and
`game-action-reflection`. Natural wording is fine inside those skills; actions
outside that skill set are not valid for this fixture.

Select the play language at launch with `--lang en` or `--lang zh`; interactive
launches prompt before the TUI starts when no language is supplied. Use
`/skill rule-repeat` or `/game rules` to show the how-to-play guide again during
play.

The save starts with a small plot frame: premise, background, opening conflict,
cast, and the live exchange with 绫波丽 (Ayanami Rei). It also tracks progress as a story graph
with an emotional reconciliation style profile. `story.branches.mainline.head`
points at the active beat, and `TURN_LOG.jsonl` records committed turns.

`content/backstory.md` contains the full demo background: how the player and
绫波丽 (Ayanami Rei) became close in Tokyo, the private promise they made, the joke and weeks
of avoidance that hurt her, and why recalling the past only helps when it turns
into accountability.

The save also declares `facts.fact_gate.rules`. Flexible chat and movement are
allowed inside their action skills, but new biology, identity, family, legal,
location, or backstory facts must pass `game_fact_check` before they can enter
narration or committed state.

The default save includes `AGENTS.json` with three restartable processors:
`state`, `plot`, and `dialogue_girlfriend`. The `dialogue` driver role expands
to the active NPC, so 绫波丽 (Ayanami Rei) is handled by her own scoped pack with
`skills/npc/girlfriend/SKILL.md`, scene details, backstory, and current save
facts rather than by a generic dialogue helper. Runtime spawns must name that
pack with `game_agent_spawn` (`pack` or `role`: `dialogue_girlfriend`) before
using the child proposal for Rei's live dialogue or reaction.
