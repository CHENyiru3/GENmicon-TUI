use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::interaction::{build_playbook, format_playbook, format_scene_frame};

const DEFAULT_SCENE_RATIO_COLS: u16 = 4;
const DEFAULT_SCENE_RATIO_ROWS: u16 = 3;
const DEFAULT_SCENE_ART_COLS: u16 = 120;
const DEFAULT_SCENE_ART_ROWS: u16 = 50;
const DEFAULT_FIGURE_RATIO_COLS: u16 = 1;
const DEFAULT_FIGURE_RATIO_ROWS: u16 = 1;
const DEFAULT_REACTION_COLS: u16 = 120;
const DEFAULT_REACTION_ROWS: u16 = 60;
const DEFAULT_REACTION_RATIO_COLS: u16 = 2;
const DEFAULT_REACTION_RATIO_ROWS: u16 = 1;

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
    Figure,
    Items,
    Status,
    Tasks,
    Briefing,
    Cast,
    Dialogue,
    Actions,
    Story,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiArtFrame {
    pub cols: u16,
    pub rows: u16,
    pub lines: Vec<String>,
    pub ratio_cols: u16,
    pub ratio_rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiArtSource {
    pub path: String,
    pub emotion: String,
    pub label: String,
    pub cols: u16,
    pub rows: u16,
    pub ratio_cols: u16,
    pub ratio_rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameMusicCueSnapshot {
    pub active_cue: Option<String>,
    pub scene_cue: Option<String>,
    pub ending_cue: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameViewSnapshot {
    pub revision: u64,
    pub scene_title: String,
    pub scene: String,
    pub figure_title: String,
    pub figure: String,
    pub status: Vec<String>,
    pub items: Vec<String>,
    pub tasks: Vec<String>,
    pub dialogue: Vec<String>,
    pub choices: Vec<String>,
    pub validation: String,
    #[serde(default)]
    pub scene_art: Option<AsciiArtFrame>,
    #[serde(default)]
    pub scene_art_source: Option<AsciiArtSource>,
    #[serde(default)]
    pub figure_art: Option<AsciiArtFrame>,
    #[serde(default)]
    pub figure_art_source: Option<AsciiArtSource>,
    #[serde(default)]
    pub figure_emotion: Option<String>,
    #[serde(default)]
    pub music: Option<GameMusicCueSnapshot>,
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

pub fn render_view_snapshot(state: &Value) -> GameViewSnapshot {
    let playbook = build_playbook(state);
    let scene = state.get("scene");
    let conversation = state.get("conversation");
    let world = state.get("world");
    let player = state.get("player");
    let revision = state
        .get("revision")
        .and_then(Value::as_u64)
        .unwrap_or_default();

    let scene_title = scene
        .and_then(|scene| string_at(scene, "location"))
        .or_else(|| panel_body(state, "scene").map(|_| "Scene"))
        .unwrap_or("Scene")
        .to_string();
    let scene_text = scene
        .map(format_scene_summary)
        .filter(|text| !text.trim().is_empty())
        .or_else(|| panel_body(state, "scene"))
        .unwrap_or_else(|| "No scene has been rendered yet.".to_string());

    let current_speaker = conversation
        .and_then(|conversation| string_at(conversation, "current_speaker"))
        .unwrap_or_default();
    let active_actor = active_actor(state, current_speaker);
    let figure_title = active_actor
        .and_then(|actor| string_at(actor, "name"))
        .filter(|name| !name.is_empty())
        .or_else(|| (!current_speaker.is_empty()).then_some(current_speaker))
        .unwrap_or("Active figure")
        .to_string();
    let figure_text = active_actor
        .map(format_actor_summary)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| figure_title.clone());
    let figure_emotion = select_figure_emotion(state, active_actor).unwrap_or_else(|| {
        state
            .pointer("/ui/reactions/default")
            .and_then(Value::as_str)
            .unwrap_or("neutral")
            .to_string()
    });
    let figure_art_source = reaction_source_for(state, &figure_emotion);
    let scene_art_source = scene_art_source_for(state);

    let mut status = Vec::new();
    if let Some(player) = player {
        if let Some(name) = string_at(player, "name")
            && !name.is_empty()
        {
            status.push(format!("Player: {name}"));
        }
        if let Some(stats) = player.get("stats").and_then(Value::as_object) {
            status.extend(
                stats
                    .iter()
                    .map(|(key, value)| format!("{}: {}", titleize(key), summarize_json(value))),
            );
        }
    }
    if let Some(room) = world.and_then(|world| world.get("room")) {
        status.extend(room_metrics(room));
    }
    if let Some(votes) = world.and_then(|world| world.get("votes")) {
        status.push(format!("Votes: {}", summarize_inline_object(votes)));
    }
    if let Some(panel) = panel_body(state, "status")
        && status.is_empty()
    {
        status.extend(non_empty_lines(&panel));
    }

    let mut items = Vec::new();
    if let Some(inventory) = player.and_then(|player| player.get("inventory")) {
        items.extend(string_list(inventory));
    }
    if let Some(world_items) = world.and_then(|world| world.get("items")) {
        items.extend(string_list(world_items));
    }
    if let Some(panel) = panel_body(state, "items")
        && items.is_empty()
    {
        items.extend(non_empty_lines(&panel));
    }

    let mut tasks = world
        .and_then(|world| world.get("quests"))
        .map(string_list)
        .unwrap_or_default();
    if let Some(node) = playbook.active_node.as_ref() {
        tasks.push(format!("Story: {}", node.title));
        if !node.summary.is_empty() {
            tasks.push(node.summary.clone());
        }
    }
    if let Some(panel) = panel_body(state, "tasks")
        && tasks.is_empty()
    {
        tasks.extend(non_empty_lines(&panel));
    }

    let mut dialogue = Vec::new();
    if let Some(conversation) = &playbook.conversation {
        dialogue.extend(conversation.last_exchange.iter().map(|line| {
            if line.tone.is_empty() {
                format!("{}: {}", line.speaker, line.line)
            } else {
                format!("{} [{}]: {}", line.speaker, line.tone, line.line)
            }
        }));
        if !conversation.prompt.is_empty() {
            dialogue.push(conversation.prompt.clone());
        }
    }
    if let Some(panel) = panel_body(state, "dialogue")
        && dialogue.is_empty()
    {
        dialogue.extend(non_empty_lines(&panel));
    }

    let choices = playbook
        .suggestions
        .iter()
        .enumerate()
        .map(|(index, suggestion)| {
            let mut line = format!("{}. {}", index + 1, suggestion.label);
            if !suggestion.description.is_empty() {
                line.push_str(" - ");
                line.push_str(&suggestion.description);
            }
            if !suggestion.input.is_empty() {
                line.push_str(" [");
                line.push_str(&suggestion.input);
                line.push(']');
            }
            line
        })
        .collect::<Vec<_>>();

    GameViewSnapshot {
        revision,
        scene_title,
        scene: scene_text,
        figure_title,
        figure: figure_text,
        status,
        items,
        tasks,
        dialogue,
        choices,
        validation: if playbook.warnings.is_empty() {
            "valid".to_string()
        } else {
            format!("{} warning(s)", playbook.warnings.len())
        },
        scene_art: ascii_frame_at(
            state.pointer("/ui/ascii/scene_art"),
            DEFAULT_SCENE_RATIO_COLS,
            DEFAULT_SCENE_RATIO_ROWS,
        ),
        scene_art_source,
        figure_art: ascii_frame_at(
            state.pointer("/ui/ascii/figure_art"),
            DEFAULT_FIGURE_RATIO_COLS,
            DEFAULT_FIGURE_RATIO_ROWS,
        ),
        figure_art_source,
        figure_emotion: Some(figure_emotion),
        music: music_snapshot(state.pointer("/ui/music")),
    }
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
            Some("figure") => RenderPanelKind::Figure,
            Some("items") => RenderPanelKind::Items,
            Some("status") => RenderPanelKind::Status,
            Some("tasks") => RenderPanelKind::Tasks,
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

fn panel_body(state: &Value, id_or_kind: &str) -> Option<String> {
    state
        .pointer("/ui/panels")
        .and_then(Value::as_array)?
        .iter()
        .find(|panel| {
            string_at(panel, "id") == Some(id_or_kind)
                || string_at(panel, "kind") == Some(id_or_kind)
        })
        .and_then(|panel| string_at(panel, "body"))
        .map(str::to_string)
}

fn format_scene_summary(scene: &Value) -> String {
    let mut lines = Vec::new();
    for key in [
        "summary",
        "what_happened",
        "immediate_stakes",
        "mood",
        "time",
    ] {
        if let Some(value) = string_at(scene, key)
            && !value.trim().is_empty()
        {
            lines.push(value.to_string());
        }
    }
    if let Some(sensory) = scene.get("sensory").map(string_list)
        && !sensory.is_empty()
    {
        lines.push(format!("Sensory: {}", sensory.join("; ")));
    }
    lines.join("\n")
}

fn active_actor<'a>(state: &'a Value, current_speaker: &str) -> Option<&'a Value> {
    let cast = state.get("cast").and_then(Value::as_array)?;
    if !current_speaker.is_empty()
        && let Some(actor) = cast.iter().find(|actor| {
            string_at(actor, "name").is_some_and(|name| name.eq_ignore_ascii_case(current_speaker))
                || string_at(actor, "id").is_some_and(|id| id.eq_ignore_ascii_case(current_speaker))
        })
    {
        return Some(actor);
    }
    cast.iter()
        .find(|actor| {
            string_at(actor, "id") != Some("player") && string_at(actor, "id") != Some("juror-13")
        })
        .or_else(|| cast.first())
}

fn format_actor_summary(actor: &Value) -> String {
    let mut lines = Vec::new();
    for (label, key) in [
        ("Role", "role"),
        ("Mood", "mood"),
        ("Presence", "presence"),
        ("Cue", "visible_cue"),
        ("Wants", "wants"),
        ("Last", "last_line"),
    ] {
        if let Some(value) = string_at(actor, key)
            && !value.trim().is_empty()
        {
            lines.push(format!("{label}: {value}"));
        }
    }
    lines.join("\n")
}

fn select_figure_emotion(state: &Value, active_actor: Option<&Value>) -> Option<String> {
    if let Some(emotion) = state
        .pointer("/ui/reactions/active")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return Some(emotion.to_string());
    }

    let active_node = state.pointer("/story/active_node").and_then(Value::as_str);
    match active_node {
        Some("success") => return Some("soft_affection".to_string()),
        Some("trust_repair") => return Some("gentle_happiness".to_string()),
        Some("honest_admission") => return Some("worried".to_string()),
        Some("pressure_failure") => return Some("sad".to_string()),
        _ => {}
    }

    if let Some(score) = state
        .pointer("/player/stats/relationship_score")
        .and_then(Value::as_i64)
    {
        if score >= 4 {
            return Some("soft_affection".to_string());
        }
        if score >= 2 {
            return Some("gentle_happiness".to_string());
        }
        if score <= -3 {
            return Some("angry".to_string());
        }
        if score < 0 {
            return Some("annoyed".to_string());
        }
    }

    let mut cues = String::new();
    if let Some(actor) = active_actor {
        for key in ["mood", "visible_cue", "last_line"] {
            if let Some(value) = string_at(actor, key) {
                cues.push(' ');
                cues.push_str(&value.to_ascii_lowercase());
            }
        }
    }
    if let Some(last_exchange) = state
        .pointer("/conversation/last_exchange")
        .and_then(Value::as_array)
    {
        for exchange in last_exchange.iter().rev().take(2) {
            for key in ["tone", "line"] {
                if let Some(value) = string_at(exchange, key) {
                    cues.push(' ');
                    cues.push_str(&value.to_ascii_lowercase());
                }
            }
        }
    }

    if cues.contains("affection") || cues.contains("tender") || cues.contains("fond") {
        Some("soft_affection".to_string())
    } else if cues.contains("cheerful")
        || cues.contains("pleased")
        || cues.contains("open smile")
        || cues.contains("brighter")
        || cues.contains("energetic")
    {
        Some("cheerful".to_string())
    } else if cues.contains("gentle")
        || cues.contains("small smile")
        || cues.contains("closed-mouth")
        || cues.contains("softened")
        || cues.contains("happy")
        || cues.contains("warm")
    {
        Some("gentle_happiness".to_string())
    } else if cues.contains("fluster") || cues.contains("caught off guard") {
        Some("flustered".to_string())
    } else if cues.contains("shy")
        || cues.contains("bashful")
        || cues.contains("blush")
        || cues.contains("embarrass")
    {
        Some("shy".to_string())
    } else if cues.contains("surpris") || cues.contains("unexpected") {
        Some("surprised".to_string())
    } else if cues.contains("confus") || cues.contains("question") {
        Some("confused".to_string())
    } else if cues.contains("worr") || cues.contains("concern") || cues.contains("nervous") {
        Some("worried".to_string())
    } else if cues.contains("tear") || cues.contains("hurt") {
        Some("teary".to_string())
    } else if cues.contains("sad")
        || cues.contains("withdraw")
        || cues.contains("downcast")
        || cues.contains("leaving")
    {
        Some("sad".to_string())
    } else if cues.contains("angry") || cues.contains("stern") {
        Some("angry".to_string())
    } else if cues.contains("annoy") || cues.contains("displeased") {
        Some("annoyed".to_string())
    } else if cues.contains("tired") || cues.contains("fatigue") || cues.contains("apathetic") {
        Some("apathetic".to_string())
    } else if cues.contains("disgust") || cues.contains("uncomfortable") {
        Some("disgusted".to_string())
    } else if cues.contains("determin") || cues.contains("resolute") {
        Some("resolute".to_string())
    } else {
        None
    }
}

fn reaction_source_for(state: &Value, emotion: &str) -> Option<AsciiArtSource> {
    let root = state.pointer("/ui/reactions/figure")?;
    let base_path = root.get("base_path").and_then(Value::as_str).unwrap_or("");
    let emotions = root.get("emotions").and_then(Value::as_object)?;
    let entry = emotions
        .get(emotion)
        .or_else(|| emotions.get("neutral"))
        .or_else(|| emotions.values().next())?;
    let file = entry
        .get("file")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?;
    let path = if base_path.trim().is_empty() {
        file.to_string()
    } else {
        format!("{}/{}", base_path.trim_end_matches('/'), file)
    };
    let selected_emotion = if emotions.contains_key(emotion) {
        emotion
    } else {
        "neutral"
    };
    Some(AsciiArtSource {
        path,
        emotion: selected_emotion.to_string(),
        label: entry
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(selected_emotion)
            .to_string(),
        cols: u16_at(entry, "cols")
            .or_else(|| u16_at(root, "cols"))
            .unwrap_or(DEFAULT_REACTION_COLS),
        rows: u16_at(entry, "rows")
            .or_else(|| u16_at(root, "rows"))
            .unwrap_or(DEFAULT_REACTION_ROWS),
        ratio_cols: entry
            .pointer("/ratio/cols")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .or_else(|| {
                root.pointer("/ratio/cols")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
            })
            .unwrap_or(DEFAULT_REACTION_RATIO_COLS)
            .max(1),
        ratio_rows: entry
            .pointer("/ratio/rows")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .or_else(|| {
                root.pointer("/ratio/rows")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
            })
            .unwrap_or(DEFAULT_REACTION_RATIO_ROWS)
            .max(1),
    })
}

fn scene_art_source_for(state: &Value) -> Option<AsciiArtSource> {
    let root = state.pointer("/ui/scene_art")?;
    let frames = root.get("frames").and_then(Value::as_object)?;
    let selected_key = root
        .get("active")
        .and_then(Value::as_str)
        .filter(|key| frames.contains_key(*key))
        .or_else(|| scene_art_key_for_active_node(state, frames))
        .or_else(|| {
            root.get("default")
                .and_then(Value::as_str)
                .filter(|key| frames.contains_key(*key))
        })?;
    let entry = frames.get(selected_key)?;
    let base_path = root.get("base_path").and_then(Value::as_str).unwrap_or("");
    let file = entry
        .get("file")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?;
    let path = if base_path.trim().is_empty() {
        file.to_string()
    } else {
        format!("{}/{}", base_path.trim_end_matches('/'), file)
    };

    Some(AsciiArtSource {
        path,
        emotion: selected_key.to_string(),
        label: entry
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(selected_key)
            .to_string(),
        cols: u16_at(entry, "cols")
            .or_else(|| u16_at(root, "cols"))
            .unwrap_or(DEFAULT_SCENE_ART_COLS),
        rows: u16_at(entry, "rows")
            .or_else(|| u16_at(root, "rows"))
            .unwrap_or(DEFAULT_SCENE_ART_ROWS),
        ratio_cols: entry
            .pointer("/ratio/cols")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .or_else(|| {
                root.pointer("/ratio/cols")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
            })
            .unwrap_or(DEFAULT_SCENE_RATIO_COLS)
            .max(1),
        ratio_rows: entry
            .pointer("/ratio/rows")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .or_else(|| {
                root.pointer("/ratio/rows")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
            })
            .unwrap_or(DEFAULT_SCENE_RATIO_ROWS)
            .max(1),
    })
}

fn scene_art_key_for_active_node<'a>(
    state: &Value,
    frames: &'a serde_json::Map<String, Value>,
) -> Option<&'a str> {
    let active_node = state
        .pointer("/story/active_node")
        .and_then(Value::as_str)?;
    if let Some(node_art) = state
        .pointer("/story/nodes")
        .and_then(Value::as_object)
        .and_then(|nodes| nodes.get(active_node))
        .and_then(|node| node.get("scene_art"))
        .and_then(Value::as_str)
        .filter(|key| frames.contains_key(*key))
    {
        return frames
            .keys()
            .find(|key| key.as_str() == node_art)
            .map(|key| key.as_str());
    }

    frames
        .iter()
        .find(|(_, frame)| {
            frame
                .get("nodes")
                .and_then(Value::as_array)
                .is_some_and(|nodes| {
                    nodes
                        .iter()
                        .filter_map(Value::as_str)
                        .any(|node| node == active_node)
                })
        })
        .map(|(key, _)| key.as_str())
}

fn room_metrics(room: &Value) -> Vec<String> {
    let Some(object) = room.as_object() else {
        return vec![summarize_json(room)];
    };
    object
        .iter()
        .map(|(key, value)| format!("{}: {}", titleize(key), summarize_json(value)))
        .collect()
}

fn summarize_inline_object(value: &Value) -> String {
    match value {
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| format!("{key} {}", summarize_json(value)))
            .collect::<Vec<_>>()
            .join(" / "),
        _ => summarize_json(value),
    }
}

fn string_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(values) => values
            .iter()
            .map(summarize_json)
            .filter(|value| !value.trim().is_empty())
            .collect(),
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| format!("{}: {}", titleize(key), summarize_json(value)))
            .filter(|value| !value.trim().is_empty())
            .collect(),
        Value::String(value) if !value.trim().is_empty() => vec![value.clone()],
        Value::Null => Vec::new(),
        other => vec![summarize_json(other)],
    }
}

fn non_empty_lines(value: &str) -> Vec<String> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn titleize(key: &str) -> String {
    let mut output = String::with_capacity(key.len());
    let mut capitalize_next = true;
    for ch in key.chars() {
        if ch == '_' || ch == '-' {
            output.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            output.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            output.push(ch);
        }
    }
    output
}

fn ascii_frame_at(
    value: Option<&Value>,
    ratio_cols: u16,
    ratio_rows: u16,
) -> Option<AsciiArtFrame> {
    let value = value?;
    if let Some(values) = value.as_array() {
        return values
            .iter()
            .find_map(|candidate| ascii_frame_from_value(candidate, ratio_cols, ratio_rows));
    }
    ascii_frame_from_value(value, ratio_cols, ratio_rows)
}

fn ascii_frame_from_value(
    value: &Value,
    ratio_cols: u16,
    ratio_rows: u16,
) -> Option<AsciiArtFrame> {
    let lines = value
        .get("lines")
        .and_then(Value::as_array)?
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let rows = u16::try_from(lines.len()).ok()?;
    if rows == 0 {
        return None;
    }
    let cols = value
        .get("cols")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or_else(|| {
            lines
                .iter()
                .map(|line| line.chars().count())
                .max()
                .and_then(|width| u16::try_from(width).ok())
                .unwrap_or(0)
        });
    if cols == 0
        || value
            .get("rows")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .is_some_and(|declared| declared != rows)
        || lines
            .iter()
            .any(|line| line.contains('\u{1b}') || line.chars().count() > usize::from(cols))
    {
        return None;
    }
    Some(AsciiArtFrame {
        cols,
        rows,
        lines,
        ratio_cols: value
            .pointer("/ratio/cols")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(ratio_cols)
            .max(1),
        ratio_rows: value
            .pointer("/ratio/rows")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(ratio_rows)
            .max(1),
    })
}

fn music_snapshot(value: Option<&Value>) -> Option<GameMusicCueSnapshot> {
    let value = value?;
    Some(GameMusicCueSnapshot {
        active_cue: optional_string_at(value, "active_cue"),
        scene_cue: optional_string_at(value, "scene_cue"),
        ending_cue: optional_string_at(value, "ending_cue"),
    })
}

fn optional_string_at(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn u16_at(value: &Value, key: &str) -> Option<u16> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
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
