use crate::app::{App, ViewMode};

use super::context::RenderContext;

pub use self::daily::build_daily_view_spec;
pub use self::filter::build_filter_view_spec;

mod daily;

pub(crate) use daily::list_content_width_for_daily;
mod filter;

pub struct ViewSpec {
    pub layout: super::layout::LayoutNode,
    pub panels: Vec<super::view_model::PanelModel>,
    pub focused_panel: Option<super::layout::PanelId>,
    pub header: super::header::HeaderModel,
}

impl ViewSpec {
    #[must_use]
    pub fn single_panel(panel: super::view_model::PanelModel) -> Self {
        let panel_id = panel.id;
        Self {
            layout: super::layout::LayoutNode::panel(panel_id),
            panels: vec![panel],
            focused_panel: Some(panel_id),
            header: super::header::HeaderModel::new(),
        }
    }
}

pub fn build_view_spec(app: &App, context: &RenderContext) -> ViewSpec {
    match app.view {
        ViewMode::Daily(_) => build_daily_view_spec(app, context),
        ViewMode::Filter(_) => build_filter_view_spec(app, context),
    }
}
