use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct RenderContext {
    pub size: Rect,
    pub header_area: Rect,
    pub main_area: Rect,
    pub footer_area: Rect,
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

        Self {
            size,
            header_area,
            main_area,
            footer_area,
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
