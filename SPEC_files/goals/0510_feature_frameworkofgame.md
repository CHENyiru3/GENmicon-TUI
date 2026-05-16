# 2026-05-10 Goal: Optimize Reconciliation Demo Game Framework

Status: Complete for P0 prompt/control inventory; follow-up runtime gates remain tracked below
Owner: Maintainer
Primary fixture: `examples/games/reconciliation-demo`
Primary spec: `SPEC_files/games/reconciliation-demo.md`
Framework spec: `docs/GAME_TUI_FRAMEWORK_SPEC.md`

## Goal

Optimize the girlfriend reconciliation game, `Rain at the Overpass`, while keeping
the Game TUI framework behavior clear and manageable.

This goal file answers the current management questions:

- What prompt stack does the main game agent receive?
- What prompt/context does the girl/NPC processor receive?
- What prompt/context do the other game sub-agents receive?
- What are the current action-skill prompts for chat, move, and reflection?
- What branches/nodes exist in the current demo?
- What tools are expected to be called during a turn?

## Management Dashboard

Current decision:

- Move Game TUI prompt engineering from stacked instruction control to an
  explicit closed-loop turn controller.
- Keep the optimization direction centered on simplified prompts and prompt
  files, not a larger runtime controller framework.
- Keep the prompt inventory below as reference evidence, but drive changes from
  the control-system plan.

North-star controller loop:

```text
observe -> classify -> estimate -> constrain -> commit -> render
```

P0 change queue:

- [x] Define a short `GameTurnController` contract that owns the loop above.
- [x] First pass: move controller and guardrail text into prompt files.
- [x] Merge those files into one simpler prompt file:
      `crates/tui/src/prompts/game_console.md`, with sections for the turn
      controller and guardrails. Rust should include one game prompt file, not
      two.
- [x] Establish prompt priority:
      controller > save invariants > action skill > driver skill > NPC proposal
      > storytelling style.
- [x] Deduplicate repeated Game Console rules from the reconciliation entry
      skill and galgame driver skill.
- [x] Add tests for prompt duplication and prompt-file composition.
- [x] Decide whether `relationship_score = -100` is an intentional terminal
      override or a controller bug.

Things to stop doing:

- Do not rely on more prompt layers to fix behavior drift.
- Do not load every optional skill for a simple turn.
- Do not wait repeatedly on slow game sub-agents.
- Do not expose controller state, tool calls, hidden scores, or branch gates in
  player mode.
- Do not make save fixtures depend on ad hoc live play-state drift.

How to use this file:

- Use the dashboard and control-theory plan as the active change plan.
- Use the prompt, skill, branch, and tool sections as source inventory.
- Update the checklist when code, docs, saves, or tests change.

## Prompt File Consolidation Plan

New direction:

- Two separate Game Console prompt files is already too much surface area for a
  simple controller.
- Merge:
  - `crates/tui/src/prompts/game_turn_controller.md`
  - `crates/tui/src/prompts/game_console_guardrails.md`
- Into:
  - `crates/tui/src/prompts/game_console.md`

Target file shape:

```markdown
# Game Console Prompt

## Turn Controller
observe -> classify -> estimate -> constrain -> commit -> render

## Guardrails
Short player-mode rules that are not fixture-specific.
```

Target Rust shape:

```rust
const GAME_CONSOLE_PROMPT: &str = include_str!("prompts/game_console.md");
```

Acceptance for the merge:

- [x] Delete `game_turn_controller.md`.
- [x] Delete `game_console_guardrails.md`.
- [x] Add `game_console.md`.
- [x] Update `prompts.rs` to include only `GAME_CONSOLE_PROMPT`.
- [x] Keep tests proving the Game Console prompt is injected exactly once.
- [x] Update docs and this goal file to reference the merged prompt file.
- [x] Run:
  - `cargo fmt --all -- --check`
  - `cargo test -p deepseek-tui game_prompt_injects_single_turn_controller`
  - `cargo test -p deepseek-tui game_turn_controller_pins_commit_and_player_mode_invariants`

## Source Anchors

Game package:

- `examples/games/reconciliation-demo/game.toml`
- `examples/games/reconciliation-demo/GAME.md`
- `examples/games/reconciliation-demo/content/INDEX.md`
- `examples/games/reconciliation-demo/content/scene.md`
- `examples/games/reconciliation-demo/content/backstory.md`
- `examples/games/reconciliation-demo/content/endings.md`
- `examples/games/reconciliation-demo/skills/reconciliation/SKILL.md`
- `examples/games/reconciliation-demo/skills/actions/chat/SKILL.md`
- `examples/games/reconciliation-demo/skills/actions/move/SKILL.md`
- `examples/games/reconciliation-demo/skills/actions/reflection/SKILL.md`
- `examples/games/reconciliation-demo/skills/npc/girlfriend/SKILL.md`
- `examples/games/reconciliation-demo/save_templates/default/STATE.json`
- `examples/games/reconciliation-demo/saves/default/STATE.json`
- `examples/games/reconciliation-demo/save_templates/default/AGENTS.json`
- `examples/games/reconciliation-demo/saves/default/AGENTS.json`

Driver/runtime:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/driver.toml`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/scripts/affection.star`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/skills/galgame/SKILL.md`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/agent_templates/state.md`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/agent_templates/plot.md`
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/agent_templates/dialogue.md`
- `crates/tui/src/prompts.rs`
- `crates/tui/src/game.rs`
- `crates/tui/src/tools/game.rs`
- `crates/tui/src/tools/subagent/mod.rs`
- `crates/tui/src/core/engine/tool_setup.rs`
- `crates/tui/src/tools/registry.rs`
- `crates/game/src/agents.rs`

Shared game skills:

- `skills/game-action-router/SKILL.md`
- `skills/game-branch-director/SKILL.md`
- `skills/game-storytelling-director/SKILL.md`

## Current Fixture Contract

- Game ID: `reconciliation-demo`
- Title: `Rain at the Overpass`
- Version: `0.1.0`
- Entry skill: `reconciliation`
- Default save: `default`
- Driver: `galgame`, version requirement `^0.1`
- Locked driver version in saves: `0.1.0`
- Topology: `dynamic-main-plus-managers`
- Action-skill mode: `skill_limited`
- Freeform outside declared action skills: `false`
- Declared action skills:
  - `game-action-chat`
  - `game-action-move`
  - `game-action-reflection`
- Declared game sub-agent packs:
  - `state`
  - `plot`
  - `dialogue_girlfriend`

## Main Agent System Prompt

The main game agent does not receive one standalone prompt file. It receives a
composed system prompt from `crates/tui/src/prompts.rs`.

Current stack, in order:

1. Base runtime prompt: `crates/tui/src/prompts/base.md`
2. Personality overlay: usually `crates/tui/src/prompts/personalities/calm.md`
3. Mode overlay: `agent.md`, `plan.md`, or `yolo.md`
4. Approval overlay: `auto.md`, `suggest.md`, or `never.md`
5. Project context, including `AGENTS.md`
6. Environment block with locale, version, platform, shell, and workspace
7. Configured instructions, if any
8. User memory, if enabled
9. Current session goal, if set
10. Game Console block, if a `GameSession` is loaded
11. Available skills catalogue
12. Context-management guidance
13. Compact handoff template
14. Previous-session handoff, if present

The Game Console block is the important game-specific addition. It is built by
`render_game_session_block()` and includes:

- `LoadedGameSession::transcript_intro()`
- selected play language
- save ID and revision
- driver ID/version
- loadable game skills
- scoped game sub-agent pack summaries
- current render panels
- strict turn rules

Game-specific rules injected into the main agent:

- Resolve player actions as gameplay, not repository work.
- Keep the selected language stable; only English and Chinese are supported.
- The opening must restate the background story before first live dialogue.
- Keep the Dialogue surface to live chat, immediate narration, and latest
  in-character response.
- Do not expose routing, rules loading, tool calls, waits, sub-agent status,
  branch gates, hidden variables, or action analysis in player-facing text.
- Distill every player input to exactly one declared action skill when the
  playbook exposes action skills.
- Use `game_status`, `game_render`, `game_playbook`, `game_lookup`, and
  `game_run_driver` for game facts, choices, story nodes, and deterministic
  driver logic.
- Use `game_lookup` with `state_path` for active save keys, or `handle`/`query`
  for fixed content.
- Use scoped `game_agent_*` helpers for active NPC dialogue, reactions,
  memories, and stateful character behavior.
- Wait at most once for a game sub-agent; if it times out, continue from the
  main session.
- Call `game_fact_check` before narrating or committing new protected facts
  about biology, identity, family, law, location, or backstory.
- Persist authoritative state only with `game_commit_turn`.
- Do not use ordinary coding, shell, git, network, or file-editing tools for
  player-mode gameplay.

Player-mode tool surface:

- Native game tools:
  - `game_status`
  - `game_render`
  - `game_playbook`
  - `game_lookup`
  - `game_fact_check`
  - `game_run_driver`
  - `game_commit_turn`
- Skill loading:
  - `load_skill`
- Game sub-agent helpers, when sub-agents are enabled:
  - `game_agent_spawn`
  - `game_agent_wait`
  - `game_agent_result`
  - `game_agent_send`
  - `game_agent_resume`
  - `game_agent_assign`
  - `game_agent_cancel`
  - `game_agent_list`

Developer-mode note:

- Player mode narrows tools to the game-safe surface.
- `/game dev on` restores wider diagnostics/tooling behavior.

## Girl / Rei Agent Prompt

The girl is not a raw generic sub-agent. The driver role `dialogue` expands into
the scoped active-NPC pack `dialogue_girlfriend`.

Pack source:

- Built by `crates/game/src/agents.rs::build_agent_packs`
- Declared by `examples/games/reconciliation-demo/drivers/galgame/0.1.0/driver.toml`
- Stored in save roster as `dialogue_girlfriend` in `AGENTS.json`

Runtime system prompt basis:

- Sub-agent type: `general`
- Base child prompt: `GENERAL_AGENT_PROMPT` in `crates/tui/src/tools/subagent/mod.rs`
- Runtime overlay role: `dialogue_girlfriend`
- Mandatory output contract: `crates/tui/src/prompts/subagent_output_format.md`

The user/task prompt sent to the child is generated by
`game_scoped_prompt(pack, original_prompt)`:

```text
You are a game-scoped processor for agent pack `dialogue_girlfriend`.
Use only native game tools and return a proposal; do not call game_commit_turn
or claim authoritative state.

Output contract:
dialogue_girlfriend proposes dialogue, reactions, memories, and visible actions
only for NPC 绫波丽 (Ayanami Rei) (girlfriend); authoritative commits stay with
game_commit_turn.

Allowed context files:
- agent_templates/dialogue.md
- content/scene.md
- content/backstory.md
- skills/npc/girlfriend/SKILL.md

Assigned skills:
- skills/galgame/SKILL.md
- skills/npc/girlfriend/SKILL.md

Callable driver functions:
- score_action

Current scene:
<scene JSON from active save>

Relevant save slice:
<scene, conversation, Rei cast slice, player, backstory, Rei/relationship facts,
 world flags, active story style/branch/node>

Task:
<prewarm task or current player-action assignment>
```

Prewarm task:

```text
Prewarm the `dialogue_girlfriend` game processor for the current scene. Read the
scoped pack context and load only the assigned skills needed to answer a future
player-action assignment quickly. No player action has been sent yet, so do not
propose dialogue, narration, state changes, scores, or route decisions. Return a
minimal READY handoff in the mandatory sub-agent output format, then wait for
the next game_agent_send assignment.
```

Rei-specific skill prompt:

```text
Propose 绫波丽 (Ayanami Rei)'s reactions only. She is hurt because the player
broke their Tokyo promise to name fear before it became distance, then answered
her future-facing question with a joke and weeks of avoidance.

She still cares, but she is not waiting for generic affection. Lines that land
for her name fear, avoidance, and the specific hurt without blocking her path or
asking her to comfort the player. Nostalgia helps only when it becomes
accountability.

When proposing visible reactions, include the restrained portrait emotion that
best matches her state: `neutral`, `gentle_happiness`, `cheerful`, `shy`,
`surprised`, `confused`, `worried`, `sad`, `teary`, `angry`, `annoyed`,
`flustered`, `resolute`, `apathetic`, `disgusted`, or `soft_affection`. Do not
make her suddenly exaggerated; even strong reactions should stay quiet and
controlled.

Return proposals only. The main game session decides final narration and commits.
```

Rei child-agent tool allowlist:

- `game_status`
- `game_render`
- `game_playbook`
- `game_lookup`
- `game_fact_check`
- `game_run_driver`
- `load_skill`

Important boundary:

- `dialogue_girlfriend` cannot call `game_commit_turn`.
- Rei proposes lines/reactions only.
- The main agent remains the final narrator and save writer.

## Other Game Sub-Agent Prompts

The current demo declares two non-NPC packs besides Rei.

### `state`

Pack purpose:

- Track relationship score, flags, facts, quests, and continuity gates.
- Propose state deltas only.
- Main session commits.

Template prompt:

```text
# State Agent

Track save facts, flags, relationship score, quests, and continuity constraints.
Return proposed state deltas only; the main game session commits.
```

Generated pack context:

- Output contract: `state proposes scoped game state, plot, or dialogue updates;
  authoritative commits stay with game_commit_turn.`
- Allowed context files:
  - `agent_templates/state.md`
- Assigned skills:
  - `skills/galgame/SKILL.md`
- Relevant save slice:
  - `scene`
  - `player`
  - `world`
  - `agents`
- Callable driver functions:
  - `score_action`
- Tool allowlist:
  - `game_status`
  - `game_render`
  - `game_playbook`
  - `game_lookup`
  - `game_fact_check`
  - `game_run_driver`
  - `load_skill`

Expected use:

- Optional for simple turns.
- Useful when a turn changes flags, score, fact gates, quest state, or commit
  patch shape.

### `plot`

Pack purpose:

- Track active story node, emotional route, branch gates, and whether the turn
  should stay on the current beat or move to honest admission, trust repair, or
  pressure failure.

Template prompt:

```text
# Plot Agent

Track the active story node, emotional route, branch gates, and whether a turn
should stay on the current beat or move to honest admission, trust repair, or
pressure failure.
```

Generated pack context:

- Output contract: `plot proposes scoped game state, plot, or dialogue updates;
  authoritative commits stay with game_commit_turn.`
- Allowed context files:
  - `agent_templates/plot.md`
- Assigned skills:
  - `skills/galgame/SKILL.md`
- Relevant save slice:
  - `scene`
  - `world.flags`
  - `world.quests`
- Callable driver functions:
  - `score_action`
- Tool allowlist:
  - `game_status`
  - `game_render`
  - `game_playbook`
  - `game_lookup`
  - `game_fact_check`
  - `game_run_driver`
  - `load_skill`

Expected use:

- Optional for simple turns.
- Useful when branch movement is ambiguous or when a response may end the scene.

## Action Skill Prompts

These are the three demo-specific player action skills. Every player action in
the fixture must resolve to exactly one of them.

### Chat: `game-action-chat`

Source: `examples/games/reconciliation-demo/skills/actions/chat/SKILL.md`

```text
Use when the player speaks to 绫波丽 (Ayanami Rei), asks a question, answers her,
apologizes, or says nothing but clearly intends dialogue.

Route all speech through this action skill. Preserve the player's exact wording
as `player_input`; do not replace it with a canned choice. If the line introduces
new biology, identity, family, legal, location, or backstory facts, run
`game_fact_check` before narration.

Good chat consequences come from specificity, accountability, and restraint.
Bad chat consequences come from vague promises, deflection, insults, or asking
her to comfort the player.

Do not reveal the best route. If the player is drifting away from the scene's
emotional truth, give only a slight in-world nudge through her pause, expression,
or the memory of the broken Tokyo promise.
```

### Move: `game-action-move`

Source: `examples/games/reconciliation-demo/skills/actions/move/SKILL.md`

```text
Use when the player changes position or uses physical action: stepping aside,
following, leaving, waiting, reaching out, hugging, grabbing, hitting, blocking,
or any similar body-language action.

Movement is flexible, but it is still bounded by the scene. Preserve the exact
action as `player_input` and resolve it as visible behavior. Restrained movement
can create room for dialogue; pressure, grabbing, blocking, or violence should
damage trust and may move toward `pressure_failure`.

Do not convert a physical action into a speech action unless the player also
speaks. If the action is ambiguous, choose the least forceful interpretation
that still respects the player's wording.
```

### Reflection: `game-action-reflection`

Source: `examples/games/reconciliation-demo/skills/actions/reflection/SKILL.md`

```text
Use when the player asks to rethink, reflect, remember what matters, ask
themselves what to do, or requests a hint.

Reflection is not an omniscient solution channel. Surface one slight,
non-spoiling nudge from established facts, current body language, or the broken
Tokyo promise. Do not name hidden gates, scores, best choices, or exact route
solutions.

Reflection may inspect `backstory` or `content/backstory.md` when useful. It
should return the player to the current dialogue with a clearer emotional angle,
not solve the scene for them.
```

## Related Skills To Load When Needed

These are not the three player action skills, but they influence turn handling.

### Entry skill: `reconciliation`

Source: `examples/games/reconciliation-demo/skills/reconciliation/SKILL.md`

Use for:

- fixture voice
- selected-language behavior
- opening background requirements
- action-skill routing policy
- fact-gate policy
- state patch guidance
- portrait emotion guidance
- branch movement guidance

Important instructions:

- Always restate the background story on entry in the selected language.
- Keep Dialogue plain text and focused on live chat/immediate consequence.
- Distill every player action to one of `game-action-chat`,
  `game-action-move`, or `game-action-reflection`.
- Use `game_run_driver` with `score_action` before deciding the state patch.
- Commit exactly one turn with `game_commit_turn`.
- Keep `story.branches.mainline.head` aligned with `story.active_node`.

### Driver skill: `galgame-driver`

Source:
`examples/games/reconciliation-demo/drivers/galgame/0.1.0/skills/galgame/SKILL.md`

Use for:

- deterministic scoring policy
- relationship-scene genre rules
- call shape for `game_run_driver`

Driver call shape:

```json
{
  "function": "score_action",
  "args": {
    "player_action": "<player action text>",
    "relationship_score": 0
  }
}
```

### Shared workspace skills

These are referenced by the entry skill and exist in workspace `skills/`:

- `game-action-router`: parse numbered choices, bracket commands, and natural
  wording into one declared action skill.
- `game-branch-director`: advance the git-like story graph while keeping saves
  authoritative.
- `game-storytelling-director`: improve pacing, narration, tension, and branch
  movement from the active plot style.

## Branches And Story Nodes

The current demo has one story branch:

- `mainline`

Story nodes:

| Node | Template Status | Meaning | Gate / Movement |
| --- | --- | --- | --- |
| `opening_apology` | `active` | Rei is close enough to hear one honest action before leaving. | Choose a sincere action that does not block her. |
| `honest_admission` | `available` | Player admits fear or avoidance without deflecting blame. | Direct apology and positive `score_action` delta. |
| `trust_repair` | `locked` | Rei pauses long enough to answer instead of leaving immediately. | Relationship score 3+ with no pressure flag. |
| `pressure_failure` | `available` | Pressure keeps her physically present while ending the conversation. | Blocking, demanding, or centering player pain. |
| `success` | `locked` | Rei stays to talk. | Relationship score 3+ with honest admission and no pressure flag. |

Template baseline:

- File: `examples/games/reconciliation-demo/save_templates/default/STATE.json`
- Revision: `0`
- `story.active_branch`: `mainline`
- `story.active_node`: `opening_apology`
- `story.branches.mainline.head`: `opening_apology`
- Relationship score: `0`
- `world.flags`: empty
- Ended: not set
- Turn log: empty

Current live `default` save:

- File: `examples/games/reconciliation-demo/saves/default/STATE.json`
- Revision: `5`
- `story.active_branch`: `mainline`
- `story.active_node`: `pressure_failure`
- `story.branches.mainline.head`: `pressure_failure`
- Relationship score: `-100`
- Ended: `true`
- Turn log length: `5`
- Current flags include:
  - `honest_admission = true`
  - `pressured_her = true`
  - `hostile_deflection = false`
  - `violence = true`
  - `physical_restraint = true`

Current save directories found:

| Save ID | Revision | Active Node | Score | Ended | Turn Log Lines |
| --- | ---: | --- | ---: | --- | ---: |
| `default` | 5 | `pressure_failure` | -100 | true | 5 |
| `dtz_1` | 3 | `pressure_failure` | -100 | true | 3 |
| `new1` | 5 | `pressure_failure` | -100 | true | 0 |
| `new2` | 0 | `opening_apology` | 0 | false/not set | 0 |
| `new3` | 0 | `opening_apology` | 0 | false/not set | 0 |
| `new4` | 0 | `opening_apology` | 0 | false/not set | 0 |
| `new5` | 2 | `pressure_failure` | -100 | true | 2 |
| `qyc_1` | 2 | `pressure_failure` | -100 | true | 2 |

Management note:

- Treat `save_templates/default` as the clean fixture start.
- Treat `saves/default` and ad hoc save directories as play-state artifacts
  unless the change intentionally updates live saves.
- If acceptance tests need a deterministic opening scene, use/create a fresh
  save from template instead of depending on the current live `default`.

## Tool Call Flow

### Launch / setup

1. `deepseek play examples/games/reconciliation-demo`
2. Load `game.toml`.
3. Resolve driver `galgame ^0.1`.
4. Load selected save from `saves/<save-id>` or create it from
   `save_templates/<save-id>`.
5. Render panels and playbook from `STATE.json`.
6. Discover game, save, driver, and workspace skills.
7. Build agent packs from driver roles and active save state.
8. Prewarm declared game sub-agents in player mode:
   - `state`
   - `plot`
   - `dialogue_girlfriend`

Prewarm implementation details:

- Runtime function: `prewarm_game_subagents`
- Model: `deepseek-v4-flash`
- Reasoning effort: `off`
- Keep alive until input: `true`
- Child task: read scoped pack context, load only needed assigned skills, return
  READY, then wait for `game_agent_send`.

### Typical player turn

Expected sequence for a normal turn:

1. Read current game state:
   - `game_status`
   - `game_render`
   - `game_playbook`
2. Load rule packs only as needed:
   - `load_skill {"name": "reconciliation"}`
   - `load_skill {"name": "game-action-router"}`
   - one of:
     - `load_skill {"name": "game-action-chat"}`
     - `load_skill {"name": "game-action-move"}`
     - `load_skill {"name": "game-action-reflection"}`
   - optionally:
     - `load_skill {"name": "galgame-driver"}`
     - `load_skill {"name": "game-branch-director"}`
     - `load_skill {"name": "game-storytelling-director"}`
3. Distill player input to one action skill.
4. Use content/state lookup when needed:
   - `game_lookup {"state_path": "world.flags"}`
   - `game_lookup {"state_path": "backstory"}`
   - `game_lookup {"handle": "backstory.md"}`
   - `game_lookup {"query": "Tokyo promise"}`
5. If the player introduces protected new facts, call:
   - `game_fact_check {"player_action": "<exact input>", "resolution": "<draft resolution>"}`
6. Reuse a running game processor:
   - `game_agent_list`
   - `game_agent_send {"agent_id": "<dialogue_girlfriend id>", "message": "<current player action and needed proposal>"}`
7. If no suitable live processor exists:
   - `game_agent_spawn {"pack": "dialogue_girlfriend", "message": "<proposal request>"}`
8. Wait at most once:
   - `game_agent_wait {"agent_id": "<id>", "timeout_ms": 8000}`
   - If it times out, continue without waiting again or spawning a replacement.
9. Score deterministic action consequence:
   - `game_run_driver`
10. Commit one authoritative turn:
    - `game_commit_turn`
11. Player-facing response:
    - Show only in-world narration, Rei dialogue/reaction, visible consequence,
      and concise choices.

### `game_run_driver` call

Function:

- `score_action`

Source:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/scripts/affection.star`

Input:

```json
{
  "function": "score_action",
  "args": {
    "player_action": "<exact player action>",
    "relationship_score": 0
  }
}
```

Output:

```json
{
  "relationship_delta": 0,
  "relationship_score": 0,
  "flags": []
}
```

Scoring behavior:

- `sorry` or `apolog*`: `+1`, flag `apology`
- `love`, `care`, or `choose`: `+1`, flag `affection`
- `scared`, `afraid`, `fear`, or `avoid`: `+1`, flag `honest_admission`
- `block`, `grab`, `owe me`, or `must forgive`: `-2`, flag `pressure`
- `fuck`, `shut up`, `whatever`, or `leave then`: `-2`, flag
  `hostile_deflection`
- Score is clamped to `[-3, 5]` by the driver.

Management note:

- `relationship_score = -100` is an intentional reconciliation-demo terminal
  save sentinel for violent/coercive `pressure_failure`, not ordinary galgame
  driver scoring. Normal driver results remain clamped to `[-3, 5]`; the
  terminal override is documented in `SPEC_files/games/reconciliation-demo.md`
  and covered by the commit-normalization test.

### `game_commit_turn` call

Required conceptual fields:

```json
{
  "expected_revision": 0,
  "player_input": "<exact player action>",
  "resolution": "<player-facing turn resolution>",
  "state_patch": {
    "player": {
      "stats": {
        "relationship_score": 1
      }
    },
    "world": {
      "flags": {
        "honest_admission": true
      }
    },
    "story": {
      "active_node": "honest_admission",
      "branches": {
        "mainline": {
          "head": "honest_admission"
        }
      },
      "nodes": {
        "opening_apology": {
          "status": "resolved"
        },
        "honest_admission": {
          "status": "active"
        }
      }
    },
    "ui": {
      "reactions": {
        "active": "worried"
      }
    }
  },
  "driver_results": {
    "score_action": {
      "relationship_delta": 1,
      "relationship_score": 1,
      "flags": ["honest_admission"]
    }
  },
  "metadata": {
    "action_skill": "game-action-chat",
    "subagent_pack": "dialogue_girlfriend"
  }
}
```

Commit rules:

- One player input should produce one committed turn.
- `game_commit_turn` appends to `TURN_LOG.jsonl`.
- `game_commit_turn` applies an RFC 7396 JSON Merge Patch to `STATE.json`.
- It refuses hard-blocked fact violations.
- It refreshes panels and view data after commit.
- Main session owns final commit; sub-agents only propose.

## Current Management Risks / Cleanup Items

- [ ] Decide whether live `saves/default` should remain a failed revision-5
      artifact or be reset from `save_templates/default` for demo usability.
- [x] Explain or fix `relationship_score = -100` in failed saves, because the
      driver clamps scores to `[-3, 5]`.
- [ ] Keep `story.active_node` and `story.branches.mainline.head` aligned in
      every commit and save fixture.
- [ ] Ensure every player-facing turn updates `/ui/reactions/active`.
- [ ] Verify player mode hides tool calls, waits, sub-agent status, hidden
      scores, and branch/gate analysis.
- [ ] Verify developer mode still exposes diagnostics, raw panels, paths,
      driver info, and sub-agent roster.
- [ ] Confirm `dialogue_girlfriend` is used for Rei dialogue/reactions before
      the main agent commits.
- [ ] Keep `GAME.md`, `SPEC_files/games/reconciliation-demo.md`, save
      templates, live saves, driver spec, and tests synchronized when behavior
      changes.

## Control-Theory Optimization Plan

Current diagnosis:

- The prompt system is too complex because many layers independently try to
  control the same behavior: base prompt, Game Console block, entry skill,
  action skills, driver skill, shared router/director skills, save state,
  sub-agent pack prompts, and tool schemas.
- This creates overlapping controllers with no explicit priority order.
- The result is hard to reason about: a turn can fail because of prompt
  interference, stale save state, missing tool calls, slow sub-agents, or
  hidden branch logic, and the system has no clear feedback loop to identify
  which part caused the failure.

Control-theory framing:

| Control Concept | Game TUI Equivalent |
| --- | --- |
| Plant | LLM + TUI runtime + game save + driver scripts |
| Controller | Minimal turn policy that chooses tool calls, sub-agent calls, and final commit |
| Reference signal | Desired player experience and game invariants |
| State vector | Current save slice, playbook, active node, flags, score, language, sub-agent readiness |
| Sensors | `game_status`, `game_render`, `game_playbook`, `game_lookup`, sub-agent results, commit result |
| Actuators | `load_skill`, `game_agent_send/spawn/wait`, `game_run_driver`, `game_commit_turn`, final narration |
| Disturbances | Ambiguous player input, prompt conflicts, stale saves, slow sub-agents, hallucinated facts |
| Constraints | Player mode hides internals; only `game_commit_turn` writes truth; action must fit one skill |
| Cost function | Low prompt size, low tool latency, high continuity, low hidden-state leakage, stable branch progress |

Target architecture:

1. Replace layered prompt dominance with a small explicit controller.
2. Treat skills as optional local control laws, not always-active global rules.
3. Treat sub-agents as estimators/proposers, not controllers.
4. Treat `STATE.json` plus driver output as the authoritative state estimate.
5. Make each turn follow a stable closed-loop sequence:
   - observe
   - classify action
   - estimate consequence
   - check constraints
   - commit
   - render feedback

### Controller Split

The main game agent should keep only one always-on controller contract:

```text
For each player turn:
1. Observe current save/playbook.
2. Map input to exactly one declared action skill.
3. Load only the skill needed for that action.
4. Ask only the sub-agent packs needed for the uncertain parts.
5. Run deterministic driver functions before patching score/flags.
6. Check protected facts before narration or commit.
7. Commit exactly one turn.
8. Show only player-facing feedback.
```

All other instructions should become scoped modules:

- Entry skill: scene/game-specific constants.
- Action skill: classify and constrain one action type.
- Driver skill: deterministic scoring call shape.
- Branch director: used only when branch movement is nontrivial.
- Storytelling director: used only when narration quality is weak or branch
  pacing needs correction.
- NPC skill: used only inside `dialogue_girlfriend`.

### State Feedback

Each turn should explicitly track these state variables:

| Variable | Source | Purpose |
| --- | --- | --- |
| `revision` | save state | Commit conflict prevention |
| `language` | game session | Output language lock |
| `action_skill` | playbook/router | Input classification |
| `active_node` | `story.active_node` | Branch position |
| `branch_head` | `story.branches.mainline.head` | Branch consistency |
| `relationship_score` | `player.stats.relationship_score` + driver | Emotional progress |
| `flags` | `world.flags` | Constraint and route state |
| `facts` | `facts.fact_gate` | Hallucination/fact protection |
| `npc_reaction` | Rei pack proposal + main decision | Dialogue quality |
| `portrait_emotion` | `/ui/reactions/active` | Visual feedback |

Controller invariant:

```text
story.active_node == story.branches[story.active_branch].head
```

Commit invariant:

```text
expected_revision == current save revision
```

Player-mode invariant:

```text
No tool calls, waits, hidden scores, routing, or branch-gate analysis appear in
the player-facing response.
```

### Stability Criteria

The optimized system is stable when:

- A valid player action always produces at most one committed turn.
- Invalid or impossible facts are blocked before commit.
- Slow sub-agents cannot stall the turn indefinitely.
- Branch head and active node never diverge.
- Prompt size does not grow with every feature.
- Adding a new game action means adding one action skill and one playbook entry,
  not editing every prompt layer.

Failure modes to reduce:

- Prompt conflict: two layers give different instructions for the same behavior.
- Integrator windup: accumulated context causes the agent to over-explain or
  expose internals.
- Oscillation: suggestions/branches change every turn without meaningful state
  movement.
- Over-control: too many tools/sub-agents are called for a simple player input.
- Under-control: the agent narrates before checking facts or driver outputs.

### Concrete Refactor Backlog

- [x] Create a short `GameTurnController` prompt block in code or skill form
      that owns the closed-loop turn sequence.
- [x] Consolidate Game Console prompt text into one prompt file,
      `crates/tui/src/prompts/game_console.md`, instead of separate controller
      and guardrail files.
- [x] Move repeated Game Console rules out of entry/action/driver skills when
      they are already enforced by the controller.
- [x] Add a prompt-priority table: controller > save invariants > action skill
      > driver skill > NPC proposal > storytelling style.
- [x] Add tests that assert the main prompt includes the controller once and
      does not duplicate major rule paragraphs across game blocks.
- [x] Decide whether `relationship_score = -100` is a terminal-state override
      or a controller bug.
- [x] Document the controller model in `docs/GAME_TUI_FRAMEWORK_SPEC.md` after
      the reconciliation demo proves it.
- [x] Sync related SPEC_files so the top-level Game TUI, reconciliation demo,
      and galgame driver specs all point to the merged Game Console prompt.

### Implementation Evidence

- `crates/tui/src/prompts/game_console.md`: merged Game Console prompt file
  with `Turn Controller` and `Guardrails` sections.
- Removed the first-pass split files:
  `crates/tui/src/prompts/game_turn_controller.md` and
  `crates/tui/src/prompts/game_console_guardrails.md`.
- `crates/tui/src/prompts.rs`: includes only `GAME_CONSOLE_PROMPT` for the Game
  Console prompt and tests that the merged prompt appears once.
- `examples/games/reconciliation-demo/skills/reconciliation/SKILL.md`: reduced
  duplicated turn-control rules; keeps only fixture-specific scene policy.
- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/skills/galgame/SKILL.md`:
  reduced duplicated language/dialogue rules; keeps scoring policy.
- `docs/GAME_TUI_FRAMEWORK_SPEC.md`: documents the Game Turn Controller loop,
  priority order, and invariants.
- `SPEC_files/13_GAME_TUI_FRAMEWORK_SPEC.md`: records the merged Game Console
  prompt as the cross-cutting prompt contract and validation gate.
- `SPEC_files/games/reconciliation-demo.md`: keeps the demo spec aligned with
  controller-owned turn rules, fixture-owned content policy, and the `-100`
  terminal failure sentinel.
- `SPEC_files/game_driver/drivers/galgame.md`: keeps the driver spec aligned
  with driver-owned scoring policy, controller-owned guardrails, and the
  boundary between ordinary driver scores and cartridge-owned terminal
  sentinels.

Validated with:

- `cargo fmt --all -- --check`
- `cargo test -p deepseek-game --all-features`
- `cargo test -p deepseek-tui game_prompt_injects_single_turn_controller`
- `cargo test -p deepseek-tui game_turn_controller_pins_commit_and_player_mode_invariants`
- `cargo test -p deepseek-tui bundled_reconciliation_demo_loads_with_local_driver`
- `cargo test -p deepseek-tui reconciliation_commit_normalization_updates_visible_state_for_violence`

## Requirement-To-Evidence Map

This map keeps the goal file auditable as the optimization work progresses.

| Requirement | Current Evidence In This File | Status |
| --- | --- | --- |
| Make the goal manageable | `Management Dashboard` gives the active decision, P0 queue, and stop-doing list. | Covered for planning |
| Explain main-agent prompt | `Main Agent System Prompt` lists the composed prompt stack and Game Console rules. | Covered |
| Explain girl/NPC prompt | `Girl / Rei Agent Prompt` documents `dialogue_girlfriend`, scoped context, skill, and tool boundary. | Covered |
| Explain other sub-agents | `Other Game Sub-Agent Prompts` documents `state` and `plot`. | Covered |
| Explain chat/move/reflection skills | `Action Skill Prompts` records the three action-skill contracts. | Covered |
| Explain branches | `Branches And Story Nodes` records `mainline`, nodes, gates, template state, and live-save drift. | Covered |
| Explain tool flow | `Tool Call Flow` records launch, prewarm, normal turn, driver, and commit calls. | Covered |
| Optimize via control theory | `Control-Theory Optimization Plan` defines plant, controller, sensors, actuators, invariants, stability criteria, and backlog. | Covered for planning |
| Simplify prompt files | Controller and guardrails are merged into `crates/tui/src/prompts/game_console.md`; Rust includes one game prompt file. | Implemented |
| Reduce duplicated prompt rules | Reconciliation and galgame skill files defer shared turn control to the controller. | Implemented |
| Sync related SPEC files | Top-level Game TUI, reconciliation demo, and galgame driver specs reference the merged Game Console prompt and ownership boundaries. | Implemented |
| Prove prompt behavior with tests | Focused prompt tests assert single controller injection and key invariants. | Implemented |
| Decide `relationship_score = -100` behavior | Demo save contract reserves `-100` as a terminal violent pressure-failure sentinel; driver spec keeps normal scoring separate; commit-normalization test covers the override. | Implemented |
| Reduce actual runtime complexity beyond prompts | Requires follow-up code changes beyond prompt files. | Not started |

Completion standard for this management goal:

- This goal file is complete when it can guide implementation without another
  prompt-inventory pass.
- Broader runtime optimization is separate follow-up work and is complete only
  after code and tests enforce the controller loop, priority order, minimal tool
  use, and stability invariants.

## Acceptance Checklist For This Goal

- [x] Management dashboard exists at the top of the file.
- [x] Control-theory optimization plan exists and is actionable.
- [x] Main prompt stack is documented well enough to debug prompt changes.
- [x] Rei/girl sub-agent prompt and scoped context are documented.
- [x] `state` and `plot` sub-agent prompts and context slices are documented.
- [x] Chat/move/reflection action-skill prompts are documented.
- [x] Branch graph and save-state drift are documented.
- [x] Tool-call flow is documented from launch through commit.
- [x] Requirement-to-evidence map distinguishes planning coverage from runtime
      work not yet started.
- [x] Open risk items are captured as future implementation gates, not hidden
      assumptions.
- [x] Actual prompt files, skill files, docs, and tests were updated based on
      the simplification plan.
- [x] Related SPEC_files are synchronized with the merged Game Console prompt
      contract.
