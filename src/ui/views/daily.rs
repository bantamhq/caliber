use ratatui::{layout::Alignment, text::Line as RatatuiLine};

use crate::app::App;

use super::super::container::ContainerConfig;
use super::super::daily::build_daily_list;
use super::super::layout::PanelId;
use super::super::theme;
use super::super::view_model::{PanelContent, PanelModel};
use super::ViewSpec;

pub fn build_daily_view_spec(
    app: &App,
    context: &super::super::context::RenderContext,
) -> ViewSpec {
    let title = RatatuiLine::from(app.current_date.format(" %m/%d/%y ").to_string())
        .alignment(Alignment::Right);
    let config = ContainerConfig {
        title: Some(title),
        border_color: theme::BORDER_DAILY,
        focused_border_color: Some(theme::BORDER_FOCUSED),
    };
    let list = build_daily_list(app, context.content_width);

    let panel_id = PanelId(0);
    let panel = PanelModel::new(panel_id, config, PanelContent::EntryList(list));

    ViewSpec::single_panel(panel)
}
