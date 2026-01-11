use crate::app::App;

use super::super::container::ContainerConfig;
use super::super::filter::build_filter_list;
use super::super::layout::PanelId;
use super::super::theme;
use super::super::view_model::{PanelContent, PanelModel};
use super::ViewSpec;

pub fn build_filter_view_spec(
    app: &App,
    context: &super::super::context::RenderContext,
) -> ViewSpec {
    let config = ContainerConfig {
        title: None,
        border_color: theme::BORDER_FILTER,
        focused_border_color: Some(theme::BORDER_FOCUSED),
    };
    let list = build_filter_list(app, context.content_width);

    let panel_id = PanelId(0);
    let panel = PanelModel::new(panel_id, config, PanelContent::EntryList(list));

    ViewSpec::single_panel(panel)
}
