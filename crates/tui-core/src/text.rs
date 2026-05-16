use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[must_use]
pub fn display_width(text: &str) -> usize {
    text.width()
}

#[must_use]
pub fn truncate_to_width(text: &str, max_width: usize) -> String {
    const ELLIPSIS: &str = "...";

    if text.width() <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    let ellipsis_width = ELLIPSIS.width();
    if max_width <= ellipsis_width {
        return ".".repeat(max_width);
    }

    let mut out = String::new();
    let mut width = 0;
    for grapheme in text.graphemes(true) {
        let grapheme_width = grapheme.width();
        if width + grapheme_width + ellipsis_width > max_width {
            break;
        }
        out.push_str(grapheme);
        width += grapheme_width;
    }
    out.push_str(ELLIPSIS);
    out
}

#[must_use]
pub fn wrap_to_width(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        wrap_words(raw_line, width, &mut lines);
    }
    lines
}

fn wrap_words(raw_line: &str, width: usize, lines: &mut Vec<String>) {
    let mut current = String::new();
    for word in raw_line.split_whitespace() {
        let separator = usize::from(!current.is_empty());
        if !current.is_empty() && current.width() + separator + word.width() <= width {
            current.push(' ');
            current.push_str(word);
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if word.width() <= width {
            current.push_str(word);
        } else {
            wrap_graphemes(word, width, lines, &mut current);
        }
    }
    if !current.is_empty() || raw_line.is_empty() {
        lines.push(current);
    }
}

fn wrap_graphemes(word: &str, width: usize, lines: &mut Vec<String>, current: &mut String) {
    let mut chunk = String::new();
    let mut chunk_width = 0;
    for grapheme in word.graphemes(true) {
        let grapheme_width = grapheme_width(grapheme);
        if chunk_width > 0 && chunk_width + grapheme_width > width {
            lines.push(std::mem::take(&mut chunk));
            chunk_width = 0;
        }
        chunk.push_str(grapheme);
        chunk_width += grapheme_width;
    }
    if !chunk.is_empty() {
        *current = chunk;
    }
}

fn grapheme_width(grapheme: &str) -> usize {
    let width = grapheme.width();
    if width == 0 {
        grapheme.chars().map(|ch| ch.width().unwrap_or(0)).sum()
    } else {
        width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_to_width_respects_wide_characters() {
        let truncated = truncate_to_width("绫波丽 waits", 8);

        assert!(truncated.width() <= 8, "{truncated}");
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn wrap_to_width_splits_words_and_long_tokens() {
        assert_eq!(
            wrap_to_width("alpha beta gammagamma", 6),
            vec!["alpha", "beta", "gammag", "amma"]
        );
    }
}
