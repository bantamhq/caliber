use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::ProjectInterfaceState;

use super::interface_popup::{PopupLayout, render_popup_frame, render_query_input};

pub fn render_project_interface(f: &mut Frame, state: &ProjectInterfaceState, area: Rect) {
    let layout = PopupLayout::new(area);

    if layout.is_too_small() {
        return;
    }

    render_popup_frame(f, &layout, "Projects");
    render_query_input(f, &layout, &state.query, true);

    let visible_height = layout.content_area.height as usize;
    let total_items = state.filtered_indices.len();
    let can_scroll_up = state.scroll_offset > 0;
    let can_scroll_down = state.scroll_offset + visible_height < total_items;

    if can_scroll_up || can_scroll_down {
        let arrows = match (can_scroll_up, can_scroll_down) {
            (true, true) => "▲▼",
            (true, false) => "▲",
            (false, true) => "▼",
            (false, false) => "",
        };
        let indicator_area = Rect {
            x: layout.query_area.x,
            y: layout.query_area.y.saturating_sub(1),
            width: layout.query_area.width,
            height: 1,
        };
        let indicator = Paragraph::new(Span::styled(arrows, Style::new().dim()))
            .alignment(Alignment::Right);
        f.render_widget(indicator, indicator_area);
    }

    let mut lines = Vec::new();
    for (i, &project_idx) in state
        .filtered_indices
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(visible_height)
    {
        let project = &state.projects[project_idx];
        let is_selected = i == state.selected;

        let indicator = if is_selected { "→" } else { " " };

        let name_style = if !project.available {
            Style::new().dim()
        } else if is_selected {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new().fg(Color::Yellow).dim()
        };

        let spans = vec![
            Span::styled(format!("{} ", indicator), Style::new().fg(Color::Blue)),
            Span::styled(project.name.clone(), name_style),
        ];

        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        let message = if state.projects.is_empty() {
            "No projects registered"
        } else {
            "No matching projects"
        };
        lines.push(Line::from(Span::styled(message, Style::new().dim())));
    }

    f.render_widget(Paragraph::new(lines), layout.content_area);
}
