use ratatui::{
    style::{Color, Style},
    text::Span,
};
use unicode_width::UnicodeWidthStr;

use crate::storage::{LATER_DATE_REGEX, NATURAL_DATE_REGEX, TAG_REGEX};

pub fn style_content(text: &str, base_style: Style, muted: bool) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut last_end = 0;

    let tag_color = if muted {
        Color::Yellow
    } else {
        Color::LightYellow
    };
    let date_color = if muted { Color::Red } else { Color::LightRed };

    let mut matches: Vec<(usize, usize, Color)> = Vec::new();

    for cap in TAG_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            matches.push((m.start(), m.end(), tag_color));
        }
    }

    for cap in LATER_DATE_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            matches.push((m.start(), m.end(), date_color));
        }
    }

    for cap in NATURAL_DATE_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            matches.push((m.start(), m.end(), date_color));
        }
    }

    matches.sort_by_key(|(start, _, _)| *start);

    for (start, end, color) in matches {
        if start > last_end {
            spans.push(Span::styled(text[last_end..start].to_string(), base_style));
        }
        spans.push(Span::styled(
            text[start..end].to_string(),
            Style::default().fg(color),
        ));
        last_end = end;
    }

    if last_end < text.len() {
        spans.push(Span::styled(text[last_end..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }

    spans
}

pub fn truncate_text(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        return text.to_string();
    }

    let ellipsis = "…";
    let target_width = max_width.saturating_sub(1); // Room for ellipsis

    let mut result = String::new();
    let mut current_width = 0;

    for ch in text.chars() {
        let ch_width = ch.to_string().width();
        if current_width + ch_width > target_width {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }

    format!("{result}{ellipsis}")
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_inclusive(' ') {
        let word_width = word.width();

        if current_width + word_width <= max_width {
            current_line.push_str(word);
            current_width += word_width;
        } else if current_line.is_empty() {
            // Word is longer than max_width, must break it by character
            for ch in word.chars() {
                let ch_width = ch.to_string().width();
                if current_width + ch_width > max_width && !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }
                current_line.push(ch);
                current_width += ch_width;
            }
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    lines
}
