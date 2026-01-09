use std::io;

use super::{App, ConfirmContext, InputMode, InterfaceContext, PromptContext, TagInterfaceState};

impl App {
    pub fn open_tag_interface(&mut self) {
        self.refresh_tag_cache();
        let tags = self.cached_journal_tags.clone();
        self.input_mode = InputMode::Interface(InterfaceContext::Tag(TagInterfaceState::new(tags)));
    }

    fn selected_tag_name(&self) -> Option<String> {
        let InputMode::Interface(InterfaceContext::Tag(ref state)) = self.input_mode else {
            return None;
        };
        state.selected_tag().map(|t| t.name.clone())
    }

    pub fn tag_interface_select(&mut self) -> io::Result<()> {
        if let Some(tag) = self.selected_tag_name() {
            self.input_mode = InputMode::Normal;
            self.quick_filter(&format!("#{tag}"))?;
        }
        Ok(())
    }

    pub fn tag_interface_delete(&mut self) {
        if let Some(tag) = self.selected_tag_name() {
            self.input_mode = InputMode::Confirm(ConfirmContext::DeleteTag(tag));
        }
    }

    pub fn tag_interface_rename(&mut self) {
        if let Some(tag) = self.selected_tag_name() {
            self.input_mode = InputMode::Prompt(PromptContext::RenameTag {
                old_tag: tag,
                buffer: crate::cursor::CursorBuffer::empty(),
            });
        }
    }

    pub fn confirm_delete_tag(&mut self, tag: &str) -> io::Result<()> {
        let result = self.delete_all_tag_occurrences(tag)?;

        self.set_status(format!("Deleted {} occurrences of #{}", result, tag));
        self.input_mode = InputMode::Normal;

        self.refresh_view_after_tag_change()?;

        Ok(())
    }

    pub fn execute_rename_tag(&mut self, old_tag: &str, new_tag: &str) -> io::Result<()> {
        let validation_error = if new_tag.is_empty() {
            Some("Tag name cannot be empty")
        } else if !new_tag
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        {
            Some("Tag must start with a letter")
        } else if !new_tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Some("Tag can only contain letters, numbers, underscore, and dash")
        } else {
            None
        };

        if let Some(error) = validation_error {
            self.set_status(error.to_string());
            self.input_mode = InputMode::Normal;
            return Ok(());
        }

        let count = self.rename_tag_occurrences(old_tag, new_tag)?;
        self.set_status(format!(
            "Renamed {count} occurrences: #{old_tag} → #{new_tag}"
        ));
        self.input_mode = InputMode::Normal;
        self.refresh_view_after_tag_change()
    }
}
