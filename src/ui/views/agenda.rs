use crate::ui::container::view_content_container_config;
use crate::ui::context::RenderContext;
use crate::ui::layout::PanelId;
use crate::ui::theme;
use crate::ui::view_model::{PanelContent, PanelModel};

use super::ViewSpec;

pub fn build_agenda_view_spec(_context: &RenderContext) -> ViewSpec {
    let config = view_content_container_config(theme::BORDER_DAILY);
    let panel = PanelModel::new(PanelId(0), config, PanelContent::Empty);
    ViewSpec::single_panel(panel)
}
