use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line as RatatuiLine,
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use super::context::RenderContext;
use super::layout::padded_content_area;
use super::model::ListModel;
use super::scroll_indicator::{ScrollIndicatorStyle, scroll_indicator_text};

pub struct ContainerConfig {
    pub title: Option<RatatuiLine<'static>>,
    pub border_color: Color,
    pub focused_border_color: Option<Color>,
}

pub struct ContainerLayout {
    pub main_area: Rect,
    pub content_area: Rect,
}

#[allow(dead_code)]
pub fn render_container(
    f: &mut Frame<'_>,
    context: &RenderContext,
    config: &ContainerConfig,
    focused: bool,
) -> ContainerLayout {
    render_container_in_area(f, context.main_area, config, focused)
}

pub fn render_container_in_area(
    f: &mut Frame<'_>,
    area: Rect,
    config: &ContainerConfig,
    focused: bool,
) -> ContainerLayout {
    let border_color = if focused {
        config.focused_border_color.unwrap_or(config.border_color)
    } else {
        config.border_color
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if let Some(title) = config.title.clone() {
        block = block.title_top(title);
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    let content_area = padded_content_area(inner);

    ContainerLayout {
        main_area: area,
        content_area,
    }
}

pub fn render_list(f: &mut Frame<'_>, list: ListModel, layout: &ContainerLayout) {
    let scroll_offset = list.scroll.offset;
    let total_lines = list.scroll.total;
    let lines = list.into_lines();

    #[allow(clippy::cast_possible_truncation)]
    let content = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
    f.render_widget(content, layout.content_area);

    let content_height = layout.content_area.height as usize;
    let can_scroll_up = scroll_offset > 0;
    let can_scroll_down = scroll_offset + content_height < total_lines;

    if let Some(arrows) = scroll_indicator_text(
        can_scroll_up,
        can_scroll_down,
        ScrollIndicatorStyle::Labeled,
    ) {
        let indicator_width = arrows.width() as u16;
        let indicator_area = Rect {
            x: layout
                .main_area
                .x
                .saturating_add(layout.main_area.width.saturating_sub(indicator_width + 1)),
            y: layout
                .main_area
                .y
                .saturating_add(layout.main_area.height.saturating_sub(1)),
            width: indicator_width,
            height: 1,
        };
        let scroll_indicator =
            Paragraph::new(ratatui::text::Span::styled(arrows, Style::default().dim()));
        f.render_widget(scroll_indicator, indicator_area);
    }
}
