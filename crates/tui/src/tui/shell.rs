use deepseek_tui_core::layout::split_vertical_shell_with_preview;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainShellProps {
    pub header_height: u16,
    pub preview_height: u16,
    pub composer_height: u16,
    pub footer_height: u16,
}

impl MainShellProps {
    #[must_use]
    pub fn new(preview_height: u16, composer_height: u16) -> Self {
        Self {
            header_height: 1,
            preview_height,
            composer_height,
            footer_height: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainShellAreas {
    pub header: Rect,
    pub chat: Rect,
    pub pending_preview: Rect,
    pub composer: Rect,
    pub footer: Rect,
}

impl MainShellAreas {
    #[must_use]
    pub fn new(area: Rect, props: MainShellProps) -> Self {
        let layout = split_vertical_shell_with_preview(
            area,
            props.header_height,
            props.preview_height,
            props.composer_height,
            props.footer_height,
        );
        Self {
            header: layout.header,
            chat: layout.body,
            pending_preview: layout.preview,
            composer: layout.composer,
            footer: layout.footer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_shell_areas_match_legacy_stack_order() {
        let areas = MainShellAreas::new(Rect::new(0, 0, 80, 24), MainShellProps::new(2, 4));

        assert_eq!(areas.header, Rect::new(0, 0, 80, 1));
        assert_eq!(areas.chat, Rect::new(0, 1, 80, 16));
        assert_eq!(areas.pending_preview, Rect::new(0, 17, 80, 2));
        assert_eq!(areas.composer, Rect::new(0, 19, 80, 4));
        assert_eq!(areas.footer, Rect::new(0, 23, 80, 1));
    }

    #[test]
    fn main_shell_areas_allow_empty_pending_preview() {
        let areas = MainShellAreas::new(Rect::new(0, 0, 80, 12), MainShellProps::new(0, 3));

        assert_eq!(areas.header, Rect::new(0, 0, 80, 1));
        assert_eq!(areas.chat, Rect::new(0, 1, 80, 7));
        assert_eq!(areas.pending_preview, Rect::new(0, 8, 80, 0));
        assert_eq!(areas.composer, Rect::new(0, 8, 80, 3));
        assert_eq!(areas.footer, Rect::new(0, 11, 80, 1));
    }
}
