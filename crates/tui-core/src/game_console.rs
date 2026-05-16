use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameConsoleLayoutMode {
    Wide,
    Medium,
    Narrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConsoleAreas {
    pub mode: GameConsoleLayoutMode,
    pub header: Rect,
    pub scene: Rect,
    pub figure: Rect,
    pub dialogue: Rect,
    pub choices: Rect,
    pub status: Rect,
    pub tasks: Rect,
    pub items: Rect,
}

#[must_use]
pub fn split_game_console(area: Rect) -> GameConsoleAreas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    let header = chunks[0];
    let body = chunks[1];

    if body.width >= 120 {
        split_wide(header, body)
    } else if body.width >= 80 {
        split_medium(header, body)
    } else {
        split_narrow(header, body)
    }
}

fn split_wide(header: Rect, body: Rect) -> GameConsoleAreas {
    let bottom_height = body.height.saturating_mul(38).saturating_div(100).max(8);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(bottom_height)])
        .split(body);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(rows[1]);
    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(26),
            Constraint::Percentage(26),
            Constraint::Percentage(14),
        ])
        .split(bottom[1]);

    GameConsoleAreas {
        mode: GameConsoleLayoutMode::Wide,
        header,
        scene: top[0],
        figure: top[1],
        dialogue: bottom[0],
        choices: side[0],
        status: side[1],
        tasks: side[2],
        items: side[3],
    }
}

fn split_medium(header: Rect, body: Rect) -> GameConsoleAreas {
    let hero_height = body.height.saturating_mul(46).saturating_div(100).max(10);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(hero_height), Constraint::Min(8)])
        .split(body);
    let hero = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(56), Constraint::Percentage(44)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[1]);
    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(28),
            Constraint::Percentage(30),
        ])
        .split(bottom[1]);

    GameConsoleAreas {
        mode: GameConsoleLayoutMode::Medium,
        header,
        scene: hero[0],
        figure: hero[1],
        dialogue: bottom[0],
        choices: side[0],
        status: side[1],
        tasks: side[2],
        items: Rect::new(side[2].x, side[2].bottom(), side[2].width, 0),
    }
}

fn split_narrow(header: Rect, body: Rect) -> GameConsoleAreas {
    let scene_height = body.height.saturating_mul(22).saturating_div(100).max(4);
    let figure_height = body.height.saturating_mul(30).saturating_div(100).max(5);
    let info_height = body.height.saturating_mul(18).saturating_div(100).max(4);
    let choice_height = body.height.saturating_mul(16).saturating_div(100).max(3);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(scene_height),
            Constraint::Length(figure_height),
            Constraint::Length(info_height),
            Constraint::Min(5),
            Constraint::Length(choice_height),
        ])
        .split(body);

    GameConsoleAreas {
        mode: GameConsoleLayoutMode::Narrow,
        header,
        scene: rows[0],
        figure: rows[1],
        status: rows[2],
        dialogue: rows[3],
        choices: rows[4],
        tasks: Rect::new(rows[2].x, rows[2].bottom(), rows[2].width, 0),
        items: Rect::new(rows[4].x, rows[4].bottom(), rows[4].width, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_console_layout_selects_representative_modes() {
        assert_eq!(
            split_game_console(Rect::new(0, 0, 140, 40)).mode,
            GameConsoleLayoutMode::Wide
        );
        assert_eq!(
            split_game_console(Rect::new(0, 0, 90, 28)).mode,
            GameConsoleLayoutMode::Medium
        );
        assert_eq!(
            split_game_console(Rect::new(0, 0, 60, 20)).mode,
            GameConsoleLayoutMode::Narrow
        );
    }

    #[test]
    fn game_console_wide_layout_keeps_dialogue_and_scene_aligned() {
        let areas = split_game_console(Rect::new(0, 0, 140, 40));

        assert_eq!(areas.header, Rect::new(0, 0, 140, 1));
        assert_eq!(areas.scene.x, areas.dialogue.x);
        assert_eq!(areas.scene.width, areas.dialogue.width);
        assert!(areas.figure.width > 0);
        assert!(areas.items.height > 0);
    }

    #[test]
    fn game_console_narrow_layout_uses_compact_info_panel() {
        let areas = split_game_console(Rect::new(0, 0, 60, 20));

        assert_eq!(areas.mode, GameConsoleLayoutMode::Narrow);
        assert!(areas.status.height > 0);
        assert_eq!(areas.tasks.height, 0);
        assert_eq!(areas.items.height, 0);
        assert!(areas.dialogue.y > areas.status.y);
    }
}
