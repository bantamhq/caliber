use ratatui::widgets::Borders;

use crate::app::App;

use super::super::container::{ContainerConfig, content_area_for};
use super::super::filter::build_filter_list;
use super::super::layout::PanelId;
use super::super::theme;
use super::super::view_model::{PanelContent, PanelModel};
use super::ViewSpec;

pub fn build_filter_view_spec(
    app: &App,
    context: &super::super::context::RenderContext,
) -> ViewSpec {
    let config = list_container_config();
    let list = build_filter_list(app, list_content_width_for_filter(context));

    let panel_id = PanelId(0);
    let panel = PanelModel::new(panel_id, config, PanelContent::EntryList(list));

    ViewSpec::single_panel(panel)
}

pub(crate) fn list_content_width_for_filter(
    context: &super::super::context::RenderContext,
) -> usize {
    list_panel_content_area(context).width as usize
}

pub(crate) fn list_content_height_for_filter(
    context: &super::super::context::RenderContext,
) -> usize {
    list_panel_content_area(context).height as usize
}

fn list_panel_content_area(
    context: &super::super::context::RenderContext,
) -> ratatui::layout::Rect {
    content_area_for(context.main_area, &list_container_config())
}

fn list_container_config() -> ContainerConfig {
    ContainerConfig {
        title: None,
        border_color: theme::BORDER_FILTER,
        focused_border_color: Some(theme::BORDER_FOCUSED),
        padded: true,
        borders: Borders::ALL,
    }
}
