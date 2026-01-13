use ratatui::style::Color;
use ratatui::text::Line as RatatuiLine;
use ratatui::widgets::Borders;

use crate::app::App;

use super::super::calendar::CalendarModel;
use super::super::container::ContainerConfig;
use super::super::daily::build_daily_list;
use super::super::layout::{LayoutNode, PanelId};
use super::super::view_model::{PanelContent, PanelModel};
use super::ViewSpec;

pub fn build_daily_view_spec(
    app: &App,
    context: &super::super::context::RenderContext,
) -> ViewSpec {
    let list_config = ContainerConfig {
        title: None,
        border_color: Color::Reset,
        focused_border_color: None,
        padded: true,
        borders: Borders::ALL,
    };
    let calendar_config = ContainerConfig {
        title: Some(RatatuiLine::from(
            app.calendar_state()
                .display_month
                .format(" %B %Y ")
                .to_string(),
        )),
        border_color: Color::Reset,
        focused_border_color: None,
        padded: false,
        borders: Borders::ALL,
    };
    let blank_config = ContainerConfig {
        title: None,
        border_color: Color::Reset,
        focused_border_color: None,
        padded: false,
        borders: Borders::ALL,
    };

    let total_width = context.main_area.width.max(1);
    let show_calendar_sidebar = app.show_calendar_sidebar();
    let calendar_width = if show_calendar_sidebar {
        CalendarModel::panel_width().min(total_width.saturating_sub(1))
    } else {
        0
    };
    let mut list_width = total_width.saturating_sub(calendar_width);
    if list_width == 0 {
        list_width = 1;
    }

    let list_content_width = list_content_width_for_daily(total_width, show_calendar_sidebar);
    let list = build_daily_list(app, list_content_width);

    let list_panel = PanelModel::new(PanelId(0), list_config, PanelContent::EntryList(list));

    if !show_calendar_sidebar {
        return ViewSpec::single_panel(list_panel);
    }

    let calendar_state = app.calendar_state();
    let calendar_model = CalendarModel {
        selected: calendar_state.selected,
        display_month: calendar_state.display_month,
        day_cache: calendar_state.day_cache.clone(),
    };

    let calendar_panel = PanelModel::new(
        PanelId(1),
        calendar_config,
        PanelContent::Calendar(calendar_model),
    );
    let blank_panel = PanelModel::new(PanelId(2), blank_config, PanelContent::Empty);

    let list_pct = ((list_width as u32) * 100 / total_width as u32) as u16;
    let calendar_pct = 100u16.saturating_sub(list_pct).max(1);

    let calendar_panel_height = CalendarModel::panel_height().min(context.main_area.height);
    let calendar_height_pct = if context.main_area.height == 0 {
        100
    } else {
        ((calendar_panel_height as u32) * 100 / context.main_area.height as u32) as u16
    };
    let calendar_height_pct = calendar_height_pct.clamp(1, 100);
    let blank_height_pct = 100u16.saturating_sub(calendar_height_pct).max(1);

    ViewSpec {
        layout: LayoutNode::row(
            vec![
                LayoutNode::panel(PanelId(0)),
                LayoutNode::column(
                    vec![LayoutNode::panel(PanelId(1)), LayoutNode::panel(PanelId(2))],
                    vec![calendar_height_pct, blank_height_pct],
                ),
            ],
            vec![list_pct.max(1), calendar_pct],
        ),
        panels: vec![list_panel, calendar_panel, blank_panel],
        focused_panel: Some(PanelId(0)),
        header: super::super::header::HeaderModel::new(),
    }
}

pub fn list_content_width_for_daily(total_width: u16, show_calendar_sidebar: bool) -> usize {
    let total_width = total_width.max(1);
    let calendar_width = if show_calendar_sidebar {
        CalendarModel::panel_width().min(total_width.saturating_sub(1))
    } else {
        0
    };
    let mut list_width = total_width.saturating_sub(calendar_width);
    if list_width == 0 {
        list_width = 1;
    }
    let sidebar_adjust = if show_calendar_sidebar { 1 } else { 0 };
    list_width.saturating_sub(4 + sidebar_adjust) as usize
}
