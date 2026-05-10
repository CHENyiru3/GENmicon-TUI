---
name: game-action-chat
description: Reconciliation demo action skill for player dialogue.
---

# Chat Action

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
