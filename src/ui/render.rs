use ratatui::Frame;

use crate::app::App;

use super::autocomplete::render_autocomplete_dropdown;
use super::calendar::render_calendar;
use super::container::{render_container_in_area, render_list};
use super::context::RenderContext;
use super::header::render_header_bar;
use super::layout::layout_nodes;
use super::overlay::{OverlayLayout, render_overlays};
use super::prep::prepare_render;
use super::scroll::set_edit_cursor;
use super::view_model::{PanelContent, build_view_model};

pub fn render_app(f: &mut Frame<'_>, app: &mut App) {
    let context = RenderContext::new(f.area());
    let prep = prepare_render(app, &context);

    let view_model = build_view_model(app, &context, prep);

    render_header_bar(f, context.header_area, view_model.header);

    let mut list_content_area = None;

    for (panel_id, rect) in layout_nodes(context.main_area, &view_model.layout) {
        if let Some(panel) = view_model.panels.get(panel_id) {
            let focused = view_model.focused_panel == Some(panel_id);
            let container_layout = render_container_in_area(f, rect, &panel.config, focused);
            if panel_id.0 == 0 {
                list_content_area = Some(container_layout.content_area);
            }
            match &panel.content {
                PanelContent::EntryList(list) => {
                    render_list(f, list.clone(), &container_layout);
                }
                PanelContent::Calendar(model) => {
                    render_calendar(f, model, container_layout.content_area);
                }
                PanelContent::Empty => {}
            }
        }
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
