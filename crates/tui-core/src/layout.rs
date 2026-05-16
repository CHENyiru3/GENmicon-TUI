use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellLayout {
    pub header: Rect,
    pub body: Rect,
    pub composer: Rect,
    pub footer: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellWithPreviewLayout {
    pub header: Rect,
    pub body: Rect,
    pub preview: Rect,
    pub composer: Rect,
    pub footer: Rect,
}

#[must_use]
pub fn split_vertical_shell(
    area: Rect,
    header_height: u16,
    composer_height: u16,
    footer_height: u16,
) -> ShellLayout {
    let header_height = header_height.min(area.height);
    let remaining_after_header = area.height.saturating_sub(header_height);
    let footer_height = footer_height.min(remaining_after_header);
    let remaining_after_footer = remaining_after_header.saturating_sub(footer_height);
    let composer_height = composer_height.min(remaining_after_footer);
    let body_height = remaining_after_footer.saturating_sub(composer_height);

    let header = Rect::new(area.x, area.y, area.width, header_height);
    let body = Rect::new(area.x, area.y + header_height, area.width, body_height);
    let composer = Rect::new(area.x, body.y + body.height, area.width, composer_height);
    let footer = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(footer_height),
        area.width,
        footer_height,
    );

    ShellLayout {
        header,
        body,
        composer,
        footer,
    }
}

#[must_use]
pub fn split_vertical_shell_with_preview(
    area: Rect,
    header_height: u16,
    preview_height: u16,
    composer_height: u16,
    footer_height: u16,
) -> ShellWithPreviewLayout {
    let header_height = header_height.min(area.height);
    let remaining_after_header = area.height.saturating_sub(header_height);
    let footer_height = footer_height.min(remaining_after_header);
    let remaining_after_footer = remaining_after_header.saturating_sub(footer_height);
    let composer_height = composer_height.min(remaining_after_footer);
    let remaining_after_composer = remaining_after_footer.saturating_sub(composer_height);
    let preview_height = preview_height.min(remaining_after_composer);
    let body_height = remaining_after_composer.saturating_sub(preview_height);

    let header = Rect::new(area.x, area.y, area.width, header_height);
    let body = Rect::new(area.x, area.y + header_height, area.width, body_height);
    let preview = Rect::new(area.x, body.y + body.height, area.width, preview_height);
    let composer = Rect::new(
        area.x,
        preview.y + preview.height,
        area.width,
        composer_height,
    );
    let footer = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(footer_height),
        area.width,
        footer_height,
    );

    ShellWithPreviewLayout {
        header,
        body,
        preview,
        composer,
        footer,
    }
}

#[must_use]
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_vertical_shell_saturates_in_small_area() {
        let layout = split_vertical_shell(Rect::new(0, 0, 20, 3), 1, 2, 1);

        assert_eq!(layout.header, Rect::new(0, 0, 20, 1));
        assert_eq!(layout.body, Rect::new(0, 1, 20, 0));
        assert_eq!(layout.composer, Rect::new(0, 1, 20, 1));
        assert_eq!(layout.footer, Rect::new(0, 2, 20, 1));
    }

    #[test]
    fn split_vertical_shell_with_preview_uses_named_regions() {
        let layout = split_vertical_shell_with_preview(Rect::new(0, 0, 80, 24), 1, 3, 4, 1);

        assert_eq!(layout.header, Rect::new(0, 0, 80, 1));
        assert_eq!(layout.body, Rect::new(0, 1, 80, 15));
        assert_eq!(layout.preview, Rect::new(0, 16, 80, 3));
        assert_eq!(layout.composer, Rect::new(0, 19, 80, 4));
        assert_eq!(layout.footer, Rect::new(0, 23, 80, 1));
    }

    #[test]
    fn split_vertical_shell_with_preview_saturates_preview_first() {
        let layout = split_vertical_shell_with_preview(Rect::new(0, 0, 20, 4), 1, 5, 2, 1);

        assert_eq!(layout.header, Rect::new(0, 0, 20, 1));
        assert_eq!(layout.body, Rect::new(0, 1, 20, 0));
        assert_eq!(layout.preview, Rect::new(0, 1, 20, 0));
        assert_eq!(layout.composer, Rect::new(0, 1, 20, 2));
        assert_eq!(layout.footer, Rect::new(0, 3, 20, 1));
    }

    #[test]
    fn centered_rect_clamps_to_parent() {
        assert_eq!(
            centered_rect(Rect::new(10, 5, 20, 10), 8, 4),
            Rect::new(16, 8, 8, 4)
        );
        assert_eq!(
            centered_rect(Rect::new(10, 5, 20, 10), 40, 20),
            Rect::new(10, 5, 20, 10)
        );
    }
}
