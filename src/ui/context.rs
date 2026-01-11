use super::footer::centered_rect;
use super::hints::HINT_OVERLAY_HEIGHT;
use super::interface_modal::POPUP_HEIGHT;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};

pub struct RenderContext {
    pub size: Rect,
    pub header_area: Rect,
    pub main_area: Rect,
    pub footer_area: Rect,
    pub content_area: Rect,
    pub content_width: usize,
    pub scroll_height: usize,
    pub help_popup_area: Rect,
    pub help_visible_height: usize,
    pub interface_visible_height: usize,
}

impl RenderContext {
    #[must_use]
    pub fn new(size: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(size);
        let header_area = chunks[0];
        let main_area = chunks[1];
        let footer_area = chunks[2];

        let inner = Block::default().borders(Borders::ALL).inner(main_area);
        let content_area = super::layout::padded_content_area(inner);

        let content_height = content_area.height as usize;
        // Hint overlay overlaps by one line, so reserve HINT_OVERLAY_HEIGHT - 1.
        let scroll_height = content_height.saturating_sub(HINT_OVERLAY_HEIGHT as usize - 1);
        let content_width = content_area.width as usize;

        let help_popup_area = centered_rect(75, 70, size);
        let help_visible_height = help_popup_area.height.saturating_sub(3) as usize;
        let interface_visible_height =
            (POPUP_HEIGHT.saturating_sub(3) as usize).min(size.height as usize);

        Self {
            size,
            header_area,
            main_area,
            footer_area,
            content_area,
            content_width,
            scroll_height,
            help_popup_area,
            help_visible_height,
            interface_visible_height,
        }
    }

    #[must_use]
    pub fn for_test(width: u16, height: u16) -> Self {
        Self::new(Rect {
            x: 0,
            y: 0,
            width,
            height,
        })
    }
}
