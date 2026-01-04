use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::HintContext;

pub const HINT_OVERLAY_HEIGHT: u16 = 5;
const COLUMN_WIDTH: usize = 16;
const MAX_COLUMNS: usize = 4;
const MAX_ITEMS: usize = 16;

pub fn render_hint_overlay(f: &mut Frame, hint_state: &HintContext, footer_area: Rect) -> bool {
    if matches!(hint_state, HintContext::Inactive) {
        return false;
    }

    let overlay_area = Rect {
        x: footer_area.x,
        y: footer_area.y.saturating_sub(HINT_OVERLAY_HEIGHT),
        width: footer_area.width,
        height: HINT_OVERLAY_HEIGHT,
    };

    if overlay_area.height == 0 || overlay_area.width < 20 {
        return false;
    }

    f.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let lines = build_hint_lines(hint_state, inner.width as usize);
    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    true
}

fn build_hint_lines(hint_state: &HintContext, width: usize) -> Vec<Line<'static>> {
    let items: Vec<String> = match hint_state {
        HintContext::Inactive => return vec![],
        HintContext::Tags { matches, .. } => {
            matches.iter().take(MAX_ITEMS).map(|t| format!("#{t}")).collect()
        }
        HintContext::Commands { matches, .. } => {
            matches.iter().take(MAX_ITEMS).map(|h| format!(":{}", h.command)).collect()
        }
        HintContext::FilterTypes { matches, .. } => {
            matches.iter().take(MAX_ITEMS).map(|h| h.syntax.to_string()).collect()
        }
        HintContext::DateOps { matches, .. } => {
            matches.iter().take(MAX_ITEMS).map(|h| h.syntax.to_string()).collect()
        }
        HintContext::Negation { matches, .. } => {
            matches.iter().take(MAX_ITEMS).map(|h| h.syntax.to_string()).collect()
        }
    };

    if items.is_empty() {
        return vec![];
    }

    let num_cols = (width / COLUMN_WIDTH).clamp(1, MAX_COLUMNS);
    let rows = items.len().div_ceil(num_cols);
    let mut lines = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut spans = Vec::new();
        for col in 0..num_cols {
            let idx = col * rows + row;
            if idx < items.len() {
                let display = format!("{:width$}", items[idx], width = COLUMN_WIDTH);
                spans.push(Span::styled(display, Style::default().fg(Color::Cyan)));
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}
