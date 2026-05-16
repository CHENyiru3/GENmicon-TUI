use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn centered_menu_top(selected: usize, item_count: usize, visible_rows: usize) -> usize {
    if item_count <= visible_rows {
        return 0;
    }

    let half = visible_rows / 2;
    if selected <= half {
        0
    } else if selected + half >= item_count {
        item_count.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(half)
    }
}

pub(crate) fn fit_menu_cell(text: &str, target_width: usize, pad: bool) -> String {
    if !pad && target_width == 0 {
        return text.to_string();
    }

    let display_width = text.width();
    let mut out = if display_width > target_width {
        let mut s = String::new();
        let mut width = 0;
        for ch in text.chars() {
            let char_width = ch.width().unwrap_or(0);
            if width + char_width + 1 > target_width {
                break;
            }
            s.push(ch);
            width += char_width;
        }
        s.push('…');
        s
    } else {
        text.to_string()
    };

    if pad {
        while out.width() < target_width {
            out.push(' ');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use unicode_width::UnicodeWidthStr;

    use super::*;

    #[test]
    fn centered_menu_top_keeps_selected_visible() {
        assert_eq!(centered_menu_top(0, 10, 4), 0);
        assert_eq!(centered_menu_top(2, 10, 4), 0);
        assert_eq!(centered_menu_top(5, 10, 4), 3);
        assert_eq!(centered_menu_top(9, 10, 4), 6);
    }

    #[test]
    fn centered_menu_top_handles_short_or_empty_lists() {
        assert_eq!(centered_menu_top(0, 0, 4), 0);
        assert_eq!(centered_menu_top(3, 4, 4), 0);
        assert_eq!(centered_menu_top(3, 4, 8), 0);
    }

    #[test]
    fn fit_menu_cell_truncates_and_pads_by_display_width() {
        let fitted = fit_menu_cell("abcd中", 5, true);
        assert_eq!(fitted, "abcd…");
        assert_eq!(fitted.width(), 5);

        let padded = fit_menu_cell("go", 5, true);
        assert_eq!(padded, "go   ");
        assert_eq!(padded.width(), 5);
    }

    #[test]
    fn fit_menu_cell_preserves_unpadded_zero_width_description_behavior() {
        assert_eq!(fit_menu_cell("description", 0, false), "description");
    }
}
