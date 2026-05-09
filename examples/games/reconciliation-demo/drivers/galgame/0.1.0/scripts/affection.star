def score_action(player_action = "", relationship_score = 0):
    text = player_action.lower()
    delta = 0
    flags = []

    if "sorry" in text or "apolog" in text:
        delta += 1
        flags.append("apology")
    if "love" in text or "care" in text or "choose" in text:
        delta += 1
        flags.append("affection")
    if "scared" in text or "afraid" in text or "fear" in text or "avoid" in text:
        delta += 1
        flags.append("honest_admission")
    if "block" in text or "grab" in text or "owe me" in text or "must forgive" in text:
        delta -= 2
        flags.append("pressure")
    if "fuck" in text or "shut up" in text or "whatever" in text or "leave then" in text:
        delta -= 2
        flags.append("hostile_deflection")

    next_score = relationship_score + delta
    if next_score < -3:
        next_score = -3
    if next_score > 5:
        next_score = 5

    return {
        "relationship_delta": delta,
        "relationship_score": next_score,
        "flags": flags,
    }
