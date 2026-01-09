use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::TagInterfaceState;

use super::interface_popup::{
    PopupLayout, render_popup_frame, render_query_input, render_scroll_indicators,
};

pub fn render_tag_interface(f: &mut Frame, state: &TagInterfaceState, area: Rect) {
    let layout = PopupLayout::new(area);

    if layout.is_too_small() {
        return;
    }

    render_popup_frame(f, &layout, "Tags");
    render_query_input(f, &layout, &state.query, state.input_focused);

    let visible_height = layout.content_area.height as usize;
    let total_items = state.filtered_indices.len();

    // Render scroll indicators using the shared function
    render_scroll_indicators(f, &layout, state.scroll_offset, visible_height, total_items);

    let mut lines = Vec::new();
    for (i, &tag_idx) in state
        .filtered_indices
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(visible_height)
    {
        let tag = &state.tags[tag_idx];
        let is_selected = i == state.selected;

        let indicator = if is_selected { "→" } else { " " };
        let indicator_style = if is_selected && !state.input_focused {
            Style::new().fg(Color::Blue)
        } else if is_selected && state.input_focused {
            Style::new().fg(Color::Blue).dim()
        } else {
            Style::new()
        };

        let name_style = if is_selected && !state.input_focused {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new().fg(Color::Yellow).dim()
        };

        let count_style = Style::new().dim();

        let spans = vec![
            Span::styled(format!("{} ", indicator), indicator_style),
            Span::styled(format!("#{}", tag.name), name_style),
            Span::styled(format!(" ({})", tag.count), count_style),
        ];

        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        let message = if state.tags.is_empty() {
            "No tags in journal"
        } else {
            "No matching tags"
        };
        lines.push(Line::from(Span::styled(message, Style::new().dim())));
    }

    f.render_widget(Paragraph::new(lines), layout.content_area);
}
