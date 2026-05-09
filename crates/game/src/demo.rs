use serde_json::{Value, json};

pub fn reconciliation_initial_state(driver_id: &str, driver_version: &str) -> Value {
    json!({
        "schema_version": 1,
        "revision": 0,
        "driver": {
            "id": driver_id,
            "version": driver_version
        },
        "scene": {
            "time": "evening",
            "location": "station overpass",
            "summary": "She is walking away because she thinks the player no longer loves her."
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
