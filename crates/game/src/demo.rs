use serde_json::{Value, json};

pub fn reconciliation_initial_state(driver_id: &str, driver_version: &str) -> Value {
    json!({
        "schema_version": 1,
        "revision": 0,
        "driver": {
            "id": driver_id,
            "version": driver_version
        },
        "plot": {
            "premise": "A relationship is about to end on a station overpass unless the player can speak honestly before she leaves.",
            "background": "For weeks the player answered fear with jokes, silence, and avoidance. She took that as proof that love had already faded.",
            "opening_conflict": "She has decided to stop waiting for an answer. The player has one chance to choose honesty over self-protection.",
            "player_role": "the partner who hurt her by withdrawing",
            "genre": "emotional reconciliation"
        },
        "scene": {
            "time": "evening",
            "location": "station overpass",
            "summary": "She is walking away because she thinks the player no longer loves her.",
            "what_happened": "A final conversation collapsed into silence; she turned toward the stairs before the player found the courage to answer.",
            "immediate_stakes": "If the player pressures her or stays vague, she leaves. If the player is specific and restrained, she may stop long enough to talk.",
            "mood": "rain, hurt restraint, one fragile opening",
            "sensory": [
                "train lights dragging across wet concrete",
                "rain ticking on the metal roof",
                "her hand tightening around the stair rail"
            ]
        },
        "cast": [
            {
                "id": "player",
                "name": "You",
                "role": "player character",
                "relationship": "her partner",
                "presence": "a few steps behind her on the overpass",
                "mood": "scared, guilty, still attached",
                "visible_cue": "your first words keep catching in your throat",
                "wants": "to make her believe the silence was fear, not indifference",
                "fear": "that honesty will arrive too late",
                "can_talk": true
            },
            {
                "id": "girlfriend",
                "name": "Mina",
                "role": "the person leaving",
                "relationship": "girlfriend",
                "presence": "one step from the stairs",
                "mood": "hurt, tired, trying not to look back",
                "visible_cue": "she pauses when the player says her name, but does not turn fully around",
                "wants": "proof that the player can name the hurt instead of hiding from it",
                "fear": "being talked into staying without anything changing",
                "last_line": "If you had something real to say, you would have said it before now.",
                "can_talk": true
            }
        ],
        "conversation": {
            "current_speaker": "Mina",
            "prompt": "She is waiting for a sentence that proves the player understands what happened.",
            "available_topics": [
                "why the player went silent",
                "what she needed that night",
                "whether love is still present",
                "letting her leave without blocking her"
            ],
            "last_exchange": [
                {
                    "speaker": "Mina",
                    "tone": "quiet, almost finished",
                    "line": "I cannot keep guessing whether I matter to you."
                },
                {
                    "speaker": "You",
                    "tone": "unspoken",
                    "line": "The answer is there, but fear is still in the way."
                }
            ]
        },
        "player": {
            "name": "Player",
            "stats": {
                "relationship_score": 0
            },
            "inventory": []
        },
        "world": {
            "flags": {},
            "quests": ["catch up emotionally", "regain trust"],
            "actors": ["girlfriend"],
            "items": []
        },
        "interaction": {
            "mode": "choice_and_freeform",
            "freeform_allowed": true,
            "verbs": [
                {
                    "command": "[APOLOGIZE]",
                    "label": "Apologize",
                    "description": "Own what you did without asking her to comfort you."
                },
                {
                    "command": "[ASK]",
                    "label": "Ask",
                    "description": "Invite her to say what still hurts."
                },
                {
                    "command": "[WAIT]",
                    "label": "Wait",
                    "description": "Let silence and body language matter."
                }
            ],
            "suggestions": [
                {
                    "id": "choice_apologize",
                    "label": "Admit fear",
                    "input": "[APOLOGIZE] I was scared and I made you feel unwanted. I still care about you.",
                    "description": "Best route toward honest admission.",
                    "target_node": "honest_admission"
                },
                {
                    "id": "choice_ask",
                    "label": "Ask what she needed",
                    "input": "[ASK] What did you need from me that night?",
                    "description": "Slower trust-repair route.",
                    "target_node": "trust_repair"
                },
                {
                    "id": "choice_stop",
                    "label": "Stop her",
                    "input": "[WAIT] I step aside so she can leave if she wants, then say one clear sentence.",
                    "description": "Shows restraint before speaking.",
                    "target_node": "trust_repair"
                }
            ]
        },
        "story": {
            "style": {
                "id": "emotional_reconciliation",
                "title": "Emotional reconciliation",
                "pacing": "Slow down for apology, silence, and visible restraint.",
                "turn_shape": "Player action -> emotional landing -> boundary check -> trust shift.",
                "branch_policy": "Branch by emotional posture: honest repair, patient listening, or pressure failure."
            },
            "active_branch": "mainline",
            "active_node": "opening_apology",
            "branches": {
                "mainline": {
                    "head": "opening_apology"
                }
            },
            "nodes": {
                "opening_apology": {
                    "title": "The last stair",
                    "status": "active",
                    "summary": "She is close enough to hear one honest action before leaving.",
                    "gate": "Choose a sincere action that does not block her.",
                    "next": ["honest_admission", "trust_repair", "pressure_failure"]
                },
                "honest_admission": {
                    "title": "Honest admission",
                    "status": "available",
                    "summary": "The player admits fear or avoidance without deflecting blame.",
                    "gate": "A direct apology and score_action delta above zero.",
                    "parents": ["opening_apology"],
                    "next": ["trust_repair"]
                },
                "trust_repair": {
                    "title": "Trust repair",
                    "status": "locked",
                    "summary": "She pauses long enough to answer instead of leaving immediately.",
                    "gate": "Relationship score 3 or higher with no pressure flag.",
                    "parents": ["opening_apology", "honest_admission"],
                    "next": ["success"]
                },
                "pressure_failure": {
                    "title": "Pressure failure",
                    "status": "available",
                    "summary": "Pressuring her may keep her physically present while ending the conversation.",
                    "gate": "Blocking, demanding, or centering the player's pain.",
                    "parents": ["opening_apology"],
                    "next": []
                },
                "success": {
                    "title": "She stays to talk",
                    "status": "locked",
                    "summary": "Trust is repaired enough for the conversation to continue off the stairs.",
                    "gate": "Relationship score 3 or higher with honest admission and no pressure flag.",
                    "parents": ["trust_repair"],
                    "next": []
                }
            }
        },
        "ui": {
            "panels": [
                {
                    "id": "scene",
                    "kind": "scene",
                    "title": "Station Overpass",
                    "body": "Rain hangs in the rail lights. She is almost at the stairs."
                },
                {
                    "id": "goal",
                    "kind": "goals",
                    "title": "Goal",
                    "body": "Be honest before she leaves."
                }
            ]
        },
        "agents": {
            "topology": "dynamic-main-plus-managers",
            "last_skill_refresh_turn": 0
        }
    })
}
