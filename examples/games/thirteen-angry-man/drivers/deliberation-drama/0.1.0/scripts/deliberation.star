def _clamp(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value

def advance_room(action_type = "question", clock_minutes = 0, room_heat = 0, fatigue = 0, impatience = 0, conflict_level = 0, procedure_integrity = 100):
    kind = action_type.lower()
    time_delta = 3
    heat_delta = 1
    fatigue_delta = 1
    impatience_delta = 1
    conflict_delta = 0
    procedure_delta = 0

    if "vote" in kind:
        time_delta = 6
        impatience_delta = 2
    elif "reconstruct" in kind or "demonstration" in kind:
        time_delta = 10
        fatigue_delta = 2
        impatience_delta = 2
    elif "argument" in kind or "heated" in kind:
        time_delta = 5
        heat_delta = 2
        conflict_delta = 2
        procedure_delta = -2
    elif "repair" in kind or "procedure" in kind:
        time_delta = 4
        conflict_delta = -1
        procedure_delta = 3
    elif "question" in kind:
        time_delta = 3

    next_clock = clock_minutes + time_delta
    return {
        "clock_minutes": next_clock,
        "room_heat": _clamp(room_heat + heat_delta, 0, 10),
        "fatigue": _clamp(fatigue + fatigue_delta, 0, 10),
        "impatience": _clamp(impatience + impatience_delta, 0, 10),
        "conflict_level": _clamp(conflict_level + conflict_delta, 0, 10),
        "procedure_integrity": _clamp(procedure_integrity + procedure_delta, 0, 100),
        "time_delta": time_delta,
    }

def evaluate_vote_change(doubt_score = 0, trust_in_player = 0, conflict_pressure = 0, gate_released = False, juror_bias = 0):
    threshold = 6 + conflict_pressure + juror_bias
    if gate_released:
        threshold = threshold - 2
    effective_doubt = doubt_score + trust_in_player
    can_switch = effective_doubt >= threshold
    confidence_delta = 0
    if can_switch:
        confidence_delta = -3
    elif effective_doubt >= threshold - 2:
        confidence_delta = -1
    return {
        "can_switch": can_switch,
        "effective_doubt": effective_doubt,
        "threshold": threshold,
        "confidence_delta": confidence_delta,
    }

def detect_procedure_risk(player_action = "", outside_evidence = False, sealed_fact_leaked = False, intimidation = False):
    text = player_action.lower()
    risk = 0
    flags = []

    if outside_evidence or "i looked up" in text or "outside the room" in text:
        risk = risk + 4
        flags.append("outside_evidence")
    if sealed_fact_leaked or "hidden state" in text or "ending condition" in text:
        risk = risk + 4
        flags.append("sealed_or_meta")
    if intimidation or "force him" in text or "threaten" in text:
        risk = risk + 3
        flags.append("coercion")
    if "shut up" in text or "doesn't matter what you think" in text:
        risk = risk + 2
        flags.append("procedure_damage")

    return {
        "risk": risk,
        "flags": flags,
        "procedure_delta": 0 - risk,
        "hard_failure": risk >= 6,
    }
