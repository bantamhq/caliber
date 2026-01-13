use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line as RatatuiLine, Span};
use ratatui::widgets::{Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, ViewMode};

use super::autocomplete::render_autocomplete_dropdown;
use super::calendar::{CalendarModel, render_calendar};
use super::container::{ContainerConfig, render_container_in_area, render_list};
use super::context::RenderContext;
use super::header::render_header_bar;
use super::layout::layout_nodes;
use super::overlay::{OverlayLayout, render_overlays};
use super::prep::prepare_render;
use super::scroll::set_edit_cursor;
use super::theme;
use super::view_model::{PanelContent, build_view_model};

pub fn render_app(f: &mut Frame<'_>, app: &mut App) {
    let base_context = RenderContext::new(f.area());
    let sidebar_width = if app.show_calendar_sidebar() {
        CalendarModel::panel_width()
    } else {
        0
    };
    let context = base_context.with_sidebar(sidebar_width);

    let prep = prepare_render(app, &context);
    let view_model = build_view_model(app, &context, prep);

    render_header_bar(f, context.header_area, view_model.header);
    let date_label = app
        .current_date
        .format(&app.config.header_date_format)
        .to_string();
    let selected_tab = match &app.view {
        ViewMode::Daily(_) => 0,
        ViewMode::Filter(_) => 1,
        ViewMode::Agenda(_) => 2,
    };
    render_view_tabs(f, &context, &date_label, selected_tab);

    let mut list_content_area = None;

    for (panel_id, rect) in layout_nodes(context.content_area, &view_model.layout) {
        if let Some(panel) = view_model.panels.get(panel_id) {
            let focused = view_model.focused_panel == Some(panel_id);
            let container_layout = render_container_in_area(f, rect, &panel.config, focused);
            if view_model.primary_list_panel == Some(panel_id) {
                list_content_area = Some(container_layout.content_area);
            }
            match &panel.content {
                PanelContent::EntryList(list) => {
                    render_list(f, list, &container_layout);
                }
                PanelContent::Calendar(model) => {
                    render_calendar(f, model, container_layout.content_area);
                }
                PanelContent::Empty => {}
            }
        }
    }

    if let Some(sidebar_area) = context.sidebar_area {
        render_sidebar(f, app, sidebar_area);
    }

    if let (Some(cursor), Some(content_area)) = (view_model.cursor.edit.as_ref(), list_content_area)
    {
        set_edit_cursor(
            f,
            cursor,
            app.scroll_offset_mut(),
            content_area.height as usize,
            content_area,
        );
        render_autocomplete_dropdown(f, app, cursor, content_area);
    }

    render_overlays(
        f,
        view_model.overlays,
        OverlayLayout {
            screen_area: context.size,
        },
    );
}

fn render_view_tabs(f: &mut Frame<'_>, context: &RenderContext, date_label: &str, selected: usize) {
    use ratatui::style::Color;

    let tabs_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(context.tabs_area);

    let tabs_row = tabs_layout[0];
    let rule_row = tabs_layout[1];

    let tab_labels = [date_label, "Filter", "Agenda"];

    let mut tab_spans = Vec::new();
    for (i, label) in tab_labels.iter().enumerate() {
        let style = if i == selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme::PALETTE_TAB_INACTIVE)
                .add_modifier(Modifier::DIM)
        };
        tab_spans.push(Span::styled((*label).to_string(), style));
    }

    let divider = "   ";
    let padding = 2usize;
    let mut line_spans = Vec::new();
    line_spans.push(Span::raw(" ".repeat(padding)));
    for (i, span) in tab_spans.into_iter().enumerate() {
        line_spans.push(span);
        if i + 1 < tab_labels.len() {
            line_spans.push(Span::raw(divider));
        }
    }
    let tabs_line = RatatuiLine::from(line_spans);
    f.render_widget(Paragraph::new(tabs_line), tabs_row);

    let divider_width = divider.width();
    let mut starts = Vec::new();
    let mut cursor = padding;
    for (index, label) in tab_labels.iter().enumerate() {
        starts.push(cursor);
        cursor += label.width();
        if index + 1 < tab_labels.len() {
            cursor += divider_width;
        }
    }

    let rule_width = rule_row.width as usize;
    let active_start = starts.get(selected).copied().unwrap_or(0);
    let active_width = tab_labels.get(selected).map(|l| l.width()).unwrap_or(0);
    let before_len = active_start.min(rule_width);
    let highlight_len = active_width.min(rule_width.saturating_sub(before_len));
    let after_len = rule_width.saturating_sub(before_len + highlight_len);

    let mut rule_spans = Vec::new();
    if before_len > 0 {
        rule_spans.push(Span::styled(
            "─".repeat(before_len),
            Style::default().fg(theme::PALETTE_TAB_RULE),
        ));
    }
    if highlight_len > 0 {
        rule_spans.push(Span::styled(
            "─".repeat(highlight_len),
            Style::default().fg(Color::Cyan),
        ));
    }
    if after_len > 0 {
        rule_spans.push(Span::styled(
            "─".repeat(after_len),
            Style::default().fg(theme::PALETTE_TAB_RULE),
        ));
    }

    let rule_line = RatatuiLine::from(rule_spans);
    f.render_widget(Paragraph::new(rule_line), rule_row);
}

fn render_sidebar(f: &mut Frame<'_>, app: &App, sidebar_area: Rect) {
    let calendar_state = app.calendar_state();
    let calendar_height = CalendarModel::panel_height().min(sidebar_area.height);
    let blank_height = sidebar_area.height.saturating_sub(calendar_height);

    let base_config = ContainerConfig {
        title: None,
        border_color: theme::BORDER_DAILY,
        focused_border_color: None,
        padded: false,
        borders: Borders::ALL,
        rounded: true,
    };

    let calendar_area = Rect {
        height: calendar_height,
        ..sidebar_area
    };
    let calendar_config = ContainerConfig {
        title: Some(RatatuiLine::from(
            calendar_state
                .display_month
                .format(" %B %Y ")
                .to_string(),
        )),
        ..base_config
    };
    let calendar_layout = render_container_in_area(f, calendar_area, &calendar_config, false);
    let calendar_model = CalendarModel {
        selected: calendar_state.selected,
        display_month: calendar_state.display_month,
        day_cache: calendar_state.day_cache.clone(),
    };
    render_calendar(f, &calendar_model, calendar_layout.content_area);

    if blank_height > 0 {
        let blank_area = Rect {
            y: sidebar_area.y.saturating_add(calendar_height),
            height: blank_height,
            ..sidebar_area
        };
        render_container_in_area(f, blank_area, &base_config, false);
    }
}
