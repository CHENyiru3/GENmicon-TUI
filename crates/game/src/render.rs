use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::interaction::{build_playbook, format_playbook, format_scene_frame};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderPanel {
    pub id: String,
    pub title: String,
    pub body: String,
    pub kind: RenderPanelKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderPanelKind {
    Scene,
    Player,
    Goals,
    Log,
    Briefing,
    Cast,
    Dialogue,
    Actions,
    Story,
    Custom,
}

pub fn render_panels(state: &Value) -> Vec<RenderPanel> {
    if let Some(panels) = state
        .pointer("/ui/panels")
        .and_then(Value::as_array)
        .filter(|panels| !panels.is_empty())
    {
        let mut rendered = panels
            .iter()
            .enumerate()
            .map(|(index, panel)| panel_from_state(index, panel))
            .collect::<Vec<_>>();
        append_playbook_panels(&mut rendered, state);
        return rendered;
    }

    let mut panels = Vec::new();
    if let Some(scene) = state.get("scene") {
        panels.push(RenderPanel {
            id: "scene".to_string(),
            title: string_at(scene, "location").unwrap_or("Scene").to_string(),
            body: string_at(scene, "summary").unwrap_or("").to_string(),
            kind: RenderPanelKind::Scene,
        });
    }
    if let Some(player) = state.get("player") {
        panels.push(RenderPanel {
            id: "player".to_string(),
            title: string_at(player, "name").unwrap_or("Player").to_string(),
            body: summarize_json(player),
            kind: RenderPanelKind::Player,
        });
    }
    if let Some(quests) = state.pointer("/world/quests") {
        panels.push(RenderPanel {
            id: "goals".to_string(),
            title: "Goals".to_string(),
            body: summarize_json(quests),
            kind: RenderPanelKind::Goals,
        });
    }
    append_playbook_panels(&mut panels, state);
    panels
}

fn panel_from_state(index: usize, panel: &Value) -> RenderPanel {
    RenderPanel {
        id: string_at(panel, "id")
            .map(str::to_string)
            .unwrap_or_else(|| format!("panel_{index}")),
        title: string_at(panel, "title")
            .map(str::to_string)
            .unwrap_or_else(|| "Panel".to_string()),
        body: string_at(panel, "body")
            .map(str::to_string)
            .unwrap_or_else(|| summarize_json(panel)),
        kind: match string_at(panel, "kind") {
            Some("scene") => RenderPanelKind::Scene,
            Some("player") => RenderPanelKind::Player,
            Some("goals") => RenderPanelKind::Goals,
            Some("log") => RenderPanelKind::Log,
            Some("briefing") => RenderPanelKind::Briefing,
            Some("cast") => RenderPanelKind::Cast,
            Some("dialogue") => RenderPanelKind::Dialogue,
            Some("actions") => RenderPanelKind::Actions,
            Some("story") => RenderPanelKind::Story,
            _ => RenderPanelKind::Custom,
        },
    }
}

fn append_playbook_panels(panels: &mut Vec<RenderPanel>, state: &Value) {
    let playbook = build_playbook(state);
    if let Some(frame) = format_scene_frame(&playbook)
        && !panels.iter().any(|panel| panel.id == "briefing")
    {
        panels.push(RenderPanel {
            id: "briefing".to_string(),
            title: "Briefing".to_string(),
            body: frame,
            kind: RenderPanelKind::Briefing,
        });
    }
    if !playbook.actors.is_empty() && !panels.iter().any(|panel| panel.id == "cast") {
        let body = playbook
            .actors
            .iter()
            .map(|actor| {
                let mut lines = vec![format!("{} - {}", actor.name, actor.role)];
                if !actor.relationship.is_empty() {
                    lines.push(format!("relationship: {}", actor.relationship));
                }
                if !actor.wants.is_empty() {
                    lines.push(format!("wants: {}", actor.wants));
                }
                if !actor.visible_cue.is_empty() {
                    lines.push(format!("cue: {}", actor.visible_cue));
                }
                lines.join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        panels.push(RenderPanel {
            id: "cast".to_string(),
            title: "Cast".to_string(),
            body,
            kind: RenderPanelKind::Cast,
        });
    }
    if let Some(conversation) = &playbook.conversation
        && !panels.iter().any(|panel| panel.id == "dialogue")
    {
        let mut lines = conversation
            .last_exchange
            .iter()
            .map(|line| {
                if line.tone.is_empty() {
                    format!("{}: \"{}\"", line.speaker, line.line)
                } else {
                    format!("{} [{}]: \"{}\"", line.speaker, line.tone, line.line)
                }
            })
            .collect::<Vec<_>>();
        if !conversation.prompt.is_empty() {
            lines.push(conversation.prompt.clone());
        }
        if !conversation.available_topics.is_empty() {
            lines.push(format!(
                "Topics: {}",
                conversation.available_topics.join(", ")
            ));
        }
        panels.push(RenderPanel {
            id: "dialogue".to_string(),
            title: "Dialogue".to_string(),
            body: lines.join("\n"),
            kind: RenderPanelKind::Dialogue,
        });
    }
    if !playbook.suggestions.is_empty() && !panels.iter().any(|panel| panel.id == "actions") {
        panels.push(RenderPanel {
            id: "actions".to_string(),
            title: "Actions".to_string(),
            body: format_playbook(&playbook),
            kind: RenderPanelKind::Actions,
        });
    }
    if let Some(node) = playbook.active_node
        && !panels.iter().any(|panel| panel.id == "story")
    {
        let mut body = Vec::new();
        body.push(format!("{} [{}]", node.title, node.status));
        if !node.summary.is_empty() {
            body.push(node.summary);
        }
        if let Some(gate) = node.gate {
            body.push(format!("Gate: {gate}"));
        }
        if !node.parents.is_empty() {
            body.push(format!("Parents: {}", node.parents.join(", ")));
        }
        if !node.next_nodes.is_empty() {
            body.push(format!("Next: {}", node.next_nodes.join(", ")));
        }
        panels.push(RenderPanel {
            id: "story".to_string(),
            title: "Story Beat".to_string(),
            body: body.join("\n"),
            kind: RenderPanelKind::Story,
        });
    }
}

fn string_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn summarize_json(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(summarize_json)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| format!("{key}: {}", summarize_json(value)))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
