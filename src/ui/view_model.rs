use crate::app::{App, InputMode, InterfaceContext};

use super::container::ContainerConfig;
use super::context::RenderContext;
use super::footer::FooterModel;
use super::header::HeaderModel;
use super::layout::{LayoutNode, PanelId};
use super::model::ListModel;
use super::overlay::{
    ConfirmModel, HelpModel, HintModel, InterfaceModel, JournalIndicatorModel, OverlayModel,
    StatusModel,
};
use super::prep::RenderPrep;
use super::scroll::CursorContext;
use super::views::build_view_spec;

pub struct ViewModel<'a> {
    pub layout: LayoutNode,
    pub panels: PanelRegistry,
    pub overlays: OverlayModel<'a>,
    pub cursor: CursorModel,
    pub header: HeaderModel,
    pub focused_panel: Option<PanelId>,
}

pub struct PanelModel {
    pub id: PanelId,
    pub config: ContainerConfig,
    pub content: PanelContent,
}

impl PanelModel {
    #[must_use]
    pub fn new(id: PanelId, config: ContainerConfig, content: PanelContent) -> Self {
        Self {
            id,
            config,
            content,
        }
    }
}

pub enum PanelContent {
    EntryList(ListModel),
}

pub struct PanelRegistry {
    panels: Vec<PanelModel>,
}

impl PanelRegistry {
    #[must_use]
    pub fn new(panels: Vec<PanelModel>) -> Self {
        Self { panels }
    }

    pub fn get(&self, id: PanelId) -> Option<&PanelModel> {
        self.panels.get(id.0)
    }
}

pub struct CursorModel {
    pub edit: Option<CursorContext>,
    pub prompt: Option<(u16, u16)>,
}

pub fn build_view_model<'a>(
    app: &'a App,
    context: &RenderContext,
    prep: RenderPrep,
) -> ViewModel<'a> {
    let current_project_id = app.current_project_id();

    let overlays = OverlayModel {
        status: StatusModel::new(app.status_message.as_deref()),
        footer: FooterModel::new(&app.view, &app.input_mode, &app.keymap),
        hint: HintModel::new(&app.hint_state),
        journal: JournalIndicatorModel::new(app.active_journal(), current_project_id.clone()),
        help: app
            .help_visible
            .then(|| HelpModel::new(&app.keymap, app.help_scroll, app.help_visible_height)),
        confirm: match &app.input_mode {
            InputMode::Confirm(confirm_context) => Some(ConfirmModel::new(confirm_context)),
            _ => None,
        },
        interface: match &app.input_mode {
            InputMode::Interface(ctx) => Some(match ctx {
                InterfaceContext::Date(state) => InterfaceModel::date(state),
                InterfaceContext::Project(state) => {
                    InterfaceModel::project(state, current_project_id.as_ref().cloned())
                }
                InterfaceContext::Tag(state) => InterfaceModel::tag(state),
            }),
            _ => None,
        },
    };

    let view_spec = build_view_spec(app, context);

    ViewModel {
        layout: view_spec.layout,
        panels: PanelRegistry::new(view_spec.panels),
        overlays,
        cursor: CursorModel {
            edit: prep.edit_cursor,
            prompt: prep.prompt_cursor,
        },
        header: view_spec.header,
        focused_panel: view_spec.focused_panel,
    }
}
