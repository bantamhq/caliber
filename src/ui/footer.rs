use std::sync::LazyLock;

use ratatui::style::Style;
use ratatui::text::Span;
use serde::Deserialize;

use crate::app::{CommandPaletteMode, InputMode, ViewMode};
use crate::dispatch::{Keymap, parse_action_id};
use crate::registry::KeyContext;

/// A single hint entry from the TOML
#[derive(Debug, Clone, Deserialize)]
struct HintDef {
    actions: Vec<String>,
    text: String,
}

/// A footer definition from the TOML
#[derive(Debug, Clone, Deserialize)]
struct FooterDef {
    hints: Vec<HintDef>,
}

/// The full footer TOML structure
#[derive(Debug, Clone, Deserialize)]
struct FooterFile {
    footer: std::collections::HashMap<String, FooterDef>,
}

static FOOTER_DATA: LazyLock<FooterFile> = LazyLock::new(|| {
    let toml_str = include_str!("../registry/footer.toml");
    toml::from_str(toml_str).expect("Failed to parse footer.toml")
});

/// Which footer to display based on current mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterMode {
    Normal,
    Edit,
    Reorder,
    Selection,
    CommandPaletteProjects,
    CommandPaletteTags,
    FilterPrompt,
}

impl FooterMode {
    /// Determine the footer mode from app state
    #[must_use]
    pub fn from_input_mode(input_mode: &InputMode, _view: &ViewMode) -> Self {
        match input_mode {
            InputMode::Normal => FooterMode::Normal,
            InputMode::Edit(_) => FooterMode::Edit,
            InputMode::Reorder => FooterMode::Reorder,
            InputMode::Selection(_) => FooterMode::Selection,
            InputMode::CommandPalette(state) => match state.mode {
                CommandPaletteMode::Commands => FooterMode::Normal, // No hints for commands
                CommandPaletteMode::Projects => FooterMode::CommandPaletteProjects,
                CommandPaletteMode::Tags => FooterMode::CommandPaletteTags,
            },
            InputMode::FilterPrompt => FooterMode::FilterPrompt,
            InputMode::Confirm(_) | InputMode::DatePicker(_) => FooterMode::Normal,
        }
    }

    /// Get the TOML key for this footer mode
    #[must_use]
    fn toml_key(self) -> &'static str {
        match self {
            FooterMode::Normal => "normal",
            FooterMode::Edit => "edit",
            FooterMode::Reorder => "reorder",
            FooterMode::Selection => "selection",
            FooterMode::CommandPaletteProjects => "command_palette_projects",
            FooterMode::CommandPaletteTags => "command_palette_tags",
            FooterMode::FilterPrompt => "filter_prompt",
        }
    }

    /// Get the KeyContext for looking up keys
    #[must_use]
    fn key_context(self) -> KeyContext {
        match self {
            FooterMode::Normal => KeyContext::DailyNormal,
            FooterMode::Edit => KeyContext::Edit,
            FooterMode::Reorder => KeyContext::Reorder,
            FooterMode::Selection => KeyContext::Selection,
            FooterMode::CommandPaletteProjects | FooterMode::CommandPaletteTags => {
                KeyContext::CommandPalette
            }
            FooterMode::FilterPrompt => KeyContext::Edit,
        }
    }
}

/// Convert a key string to display format with unicode arrows
#[must_use]
fn format_key_display(key: &str) -> String {
    match key {
        "up" => "↑".to_string(),
        "down" => "↓".to_string(),
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        "ret" | "enter" => "enter".to_string(),
        "esc" | "escape" => "esc".to_string(),
        "tab" => "tab".to_string(),
        "backtab" | "S-tab" => "shift+tab".to_string(),
        "space" => "␣".to_string(),
        "pageup" => "PgUp".to_string(),
        "pagedown" => "PgDn".to_string(),
        other => other.to_string(),
    }
}

/// A rendered hint ready for display
#[derive(Debug, Clone)]
pub struct RenderedHint {
    pub keys: String,
    pub text: String,
}

impl RenderedHint {
    /// Total display width including separators
    #[must_use]
    pub fn width(&self) -> usize {
        // "keys text  " format: key + space + text + two trailing spaces
        self.keys.chars().count() + 1 + self.text.len() + 2
    }
}

/// Build rendered hints for the given mode
#[must_use]
pub fn build_hints(mode: FooterMode, keymap: &Keymap) -> Vec<RenderedHint> {
    let context = mode.key_context();
    let toml_key = mode.toml_key();

    let Some(footer_def) = FOOTER_DATA.footer.get(toml_key) else {
        return Vec::new();
    };

    footer_def
        .hints
        .iter()
        .filter_map(|hint| {
            // Get first key for each action, join with /
            let keys: Vec<String> = hint
                .actions
                .iter()
                .filter_map(|action_str| {
                    let action_id = parse_action_id(action_str)?;
                    let keys = keymap.keys_for_action_ordered(context, action_id);
                    keys.first().map(|k| format_key_display(k))
                })
                .collect();

            if keys.is_empty() {
                return None;
            }

            Some(RenderedHint {
                keys: keys.join("/"),
                text: hint.text.clone(),
            })
        })
        .collect()
}

/// Build footer spans that fit within the given width
#[must_use]
pub fn build_footer_spans(
    hints: &[RenderedHint],
    max_width: usize,
    key_style: Style,
    text_style: Style,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut used_width = 0;

    for hint in hints {
        let hint_width = hint.width();
        if used_width + hint_width > max_width {
            break;
        }

        spans.push(Span::styled(hint.keys.clone(), key_style));
        spans.push(Span::styled(format!(" {}  ", hint.text), text_style));
        used_width += hint_width;
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_key_display() {
        assert_eq!(format_key_display("up"), "↑");
        assert_eq!(format_key_display("down"), "↓");
        assert_eq!(format_key_display("left"), "←");
        assert_eq!(format_key_display("right"), "→");
        assert_eq!(format_key_display("ret"), "enter");
        assert_eq!(format_key_display("space"), "␣");
        assert_eq!(format_key_display("a"), "a");
    }

    #[test]
    fn test_footer_data_loads() {
        // Just verify the TOML parses correctly
        let _ = &*FOOTER_DATA;
        assert!(FOOTER_DATA.footer.contains_key("normal"));
        assert!(FOOTER_DATA.footer.contains_key("edit"));
    }
}
