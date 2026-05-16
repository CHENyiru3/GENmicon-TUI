use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtCell {
    pub symbol: char,
    pub style: Style,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtFrame {
    pub ratio_cols: u16,
    pub ratio_rows: u16,
    pub lines: Vec<Vec<ArtCell>>,
}

#[must_use]
pub fn parse_ansi_art_lines(raw: &str) -> Vec<Vec<ArtCell>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut style = Style::default();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\u{1b}' => {
                if chars.peek() == Some(&'[') {
                    let _ = chars.next();
                    let mut sequence = String::new();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            if next == 'm' {
                                apply_sgr_sequence(&sequence, &mut style);
                            }
                            break;
                        }
                        sequence.push(next);
                    }
                } else {
                    skip_unsupported_escape(&mut chars);
                }
            }
            '\n' => {
                if !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                }
            }
            '\r' => {}
            symbol => row.push(ArtCell { symbol, style }),
        }
    }

    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

fn skip_unsupported_escape<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    match chars.peek().copied() {
        Some(']') => {
            let _ = chars.next();
            let mut previous_was_escape = false;
            for next in chars.by_ref() {
                if next == '\u{7}' || (previous_was_escape && next == '\\') {
                    break;
                }
                previous_was_escape = next == '\u{1b}';
            }
        }
        Some('P' | '^' | '_' | 'X') => {
            let _ = chars.next();
            let mut previous_was_escape = false;
            for next in chars.by_ref() {
                if previous_was_escape && next == '\\' {
                    break;
                }
                previous_was_escape = next == '\u{1b}';
            }
        }
        Some(_) => {
            let _ = chars.next();
        }
        None => {}
    }
}

fn apply_sgr_sequence(sequence: &str, style: &mut Style) {
    let values = if sequence.trim().is_empty() {
        vec![Some(0)]
    } else {
        sequence
            .split(';')
            .map(|part| part.parse::<u16>().ok())
            .collect::<Vec<_>>()
    };

    let mut index = 0usize;
    while index < values.len() {
        match values[index] {
            Some(0) => {
                *style = Style::default();
                index += 1;
            }
            Some(1) => {
                *style = style.add_modifier(Modifier::BOLD);
                index += 1;
            }
            Some(22) => {
                *style = style.remove_modifier(Modifier::BOLD);
                index += 1;
            }
            Some(30..=37) => {
                *style = style.fg(ansi16_color(values[index].unwrap_or(39), false));
                index += 1;
            }
            Some(40..=47) => {
                *style = style.bg(ansi16_color(values[index].unwrap_or(49) - 10, false));
                index += 1;
            }
            Some(90..=97) => {
                *style = style.fg(ansi16_color(values[index].unwrap_or(99) - 60, true));
                index += 1;
            }
            Some(100..=107) => {
                *style = style.bg(ansi16_color(values[index].unwrap_or(109) - 70, true));
                index += 1;
            }
            Some(38) | Some(48) => {
                let is_foreground = values[index] == Some(38);
                if values.get(index + 1) == Some(&Some(2))
                    && let (Some(Some(r)), Some(Some(g)), Some(Some(b))) = (
                        values.get(index + 2),
                        values.get(index + 3),
                        values.get(index + 4),
                    )
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (u8::try_from(*r), u8::try_from(*g), u8::try_from(*b))
                {
                    let color = Color::Rgb(r, g, b);
                    *style = if is_foreground {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                    index += 5;
                    continue;
                }
                if values.get(index + 1) == Some(&Some(5))
                    && let Some(Some(value)) = values.get(index + 2)
                    && let Ok(indexed) = u8::try_from(*value)
                {
                    let color = Color::Indexed(indexed);
                    *style = if is_foreground {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                    index += 3;
                    continue;
                }
                index += 1;
            }
            Some(39) => {
                *style = style.fg(Color::Reset);
                index += 1;
            }
            Some(49) => {
                *style = style.bg(Color::Reset);
                index += 1;
            }
            _ => {
                index += 1;
            }
        }
    }
}

fn ansi16_color(code: u16, bright: bool) -> Color {
    match (code, bright) {
        (30, false) => Color::Black,
        (31, false) => Color::Red,
        (32, false) => Color::Green,
        (33, false) => Color::Yellow,
        (34, false) => Color::Blue,
        (35, false) => Color::Magenta,
        (36, false) => Color::Cyan,
        (37, false) => Color::Gray,
        (30, true) => Color::DarkGray,
        (31, true) => Color::LightRed,
        (32, true) => Color::LightGreen,
        (33, true) => Color::LightYellow,
        (34, true) => Color::LightBlue,
        (35, true) => Color::LightMagenta,
        (36, true) => Color::LightCyan,
        (37, true) => Color::White,
        _ => Color::Reset,
    }
}

#[must_use]
pub fn scale_art_lines(lines: &[Vec<ArtCell>], width: u16, height: u16) -> Vec<Vec<ArtCell>> {
    let target_width = usize::from(width);
    let target_height = usize::from(height);
    if target_width == 0 || target_height == 0 || lines.is_empty() {
        return Vec::new();
    }

    let source_height = lines.len();
    let output_height = source_height.min(target_height);
    (0..output_height)
        .map(|row| {
            let source_row = row.saturating_mul(source_height) / output_height;
            scale_art_line(&lines[source_row], target_width)
        })
        .collect()
}

fn scale_art_line(line: &[ArtCell], target_width: usize) -> Vec<ArtCell> {
    if target_width == 0 {
        return Vec::new();
    }
    if line.len() <= target_width {
        return line.to_vec();
    }
    (0..target_width)
        .map(|col| {
            let source_col = col.saturating_mul(line.len()) / target_width;
            line[source_col]
        })
        .collect()
}

#[must_use]
pub fn fit_rect_to_ratio(area: Rect, ratio_cols: u16, ratio_rows: u16) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }
    let ratio_cols = u32::from(ratio_cols.max(1));
    let ratio_rows = u32::from(ratio_rows.max(1));
    let area_width = u32::from(area.width);
    let area_height = u32::from(area.height);

    let (width, height) =
        if area_width.saturating_mul(ratio_rows) > area_height.saturating_mul(ratio_cols) {
            let width = area_height
                .saturating_mul(ratio_cols)
                .saturating_div(ratio_rows)
                .max(1);
            (width.min(area_width), area_height)
        } else {
            let height = area_width
                .saturating_mul(ratio_rows)
                .saturating_div(ratio_cols)
                .max(1);
            (area_width, height.min(area_height))
        };

    Rect {
        x: area
            .x
            .saturating_add((area.width.saturating_sub(width as u16)) / 2),
        y: area
            .y
            .saturating_add((area.height.saturating_sub(height as u16)) / 2),
        width: width as u16,
        height: height as u16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_rect_to_ratio_stays_inside_area() {
        let area = Rect::new(2, 3, 40, 10);
        let fitted = fit_rect_to_ratio(area, 4, 3);
        assert!(fitted.x >= area.x);
        assert!(fitted.y >= area.y);
        assert!(fitted.right() <= area.right());
        assert!(fitted.bottom() <= area.bottom());
    }

    #[test]
    fn scale_art_lines_fits_target_size() {
        let source = (0..60)
            .map(|row| {
                (0..120)
                    .map(|col| ArtCell {
                        symbol: if (row + col) % 2 == 0 { '#' } else { '.' },
                        style: Style::default(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let scaled = scale_art_lines(&source, 37, 11);

        assert_eq!(scaled.len(), 11);
        assert!(scaled.iter().all(|line| line.len() <= 37));
    }

    #[test]
    fn ansi_art_parser_preserves_sgr_color() {
        let rows = parse_ansi_art_lines("\u{1b}[31mAB\u{1b}[0m\nCD\n");
        let rendered = rows
            .iter()
            .map(|line| line.iter().map(|cell| cell.symbol).collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(rendered, vec!["AB".to_string(), "CD".to_string()]);
        assert_eq!(rows[0][0].style.fg, Some(Color::Red));
        assert_eq!(rows[1][0].style.fg, None);
    }

    #[test]
    fn ansi_art_parser_skips_non_sgr_terminal_controls() {
        let rows =
            parse_ansi_art_lines("\u{1b}]0;window title\u{7}\u{1b}[?25l\u{1b}[31mAB\u{1b}[0m\n");
        let rendered = rows
            .iter()
            .map(|line| line.iter().map(|cell| cell.symbol).collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(rendered, vec!["AB".to_string()]);
        assert_eq!(rows[0][0].style.fg, Some(Color::Red));
    }
}
