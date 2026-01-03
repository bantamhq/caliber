use chrono::Local;

use crate::cursor::CursorBuffer;
use crate::storage::{self, Entry, EntryType, FilterEntry, Line};

use super::{App, EditContext, InputMode, InsertPosition, ViewMode};

impl App {
    /// Preprocesses content before saving: expands favorite tags and normalizes dates.
    fn preprocess_content(&self, content: &str) -> String {
        let content = storage::expand_favorite_tags(content, &self.config.favorite_tags);
        storage::normalize_natural_dates(&content, Local::now().date_naive())
    }

    /// Cycle entry type while editing (BackTab)
    pub fn cycle_edit_entry_type(&mut self) {
        match &mut self.input_mode {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                let line_idx = match self.entry_indices.get(*entry_index) {
                    Some(&idx) => idx,
                    None => return,
                };
                if let Line::Entry(entry) = &mut self.lines[line_idx] {
                    entry.entry_type = entry.entry_type.cycle();
                }
            }
            InputMode::Edit(EditContext::FilterEdit {
                date,
                line_index,
                filter_index,
            }) => {
                let date = *date;
                let line_index = *line_index;
                let filter_index = *filter_index;

                if let Ok(Some(new_type)) = storage::cycle_entry_type(date, line_index)
                    && let ViewMode::Filter(state) = &mut self.view
                    && let Some(filter_entry) = state.entries.get_mut(filter_index)
                {
                    filter_entry.entry_type = new_type;
                    filter_entry.completed =
                        matches!(filter_entry.entry_type, EntryType::Task { completed: true });
                    if date == self.current_date {
                        let _ = self.reload_current_day();
                    }
                }
            }
            InputMode::Edit(EditContext::FilterQuickAdd { entry_type, .. }) => {
                *entry_type = entry_type.cycle();
            }
            InputMode::Edit(EditContext::LaterEdit {
                source_date,
                line_index,
                later_index,
            }) => {
                let source_date = *source_date;
                let line_index = *line_index;
                let later_index = *later_index;

                if let Ok(Some(new_type)) = storage::cycle_entry_type(source_date, line_index)
                    && let ViewMode::Daily(state) = &mut self.view
                    && let Some(later_entry) = state.later_entries.get_mut(later_index)
                {
                    later_entry.entry_type = new_type;
                    later_entry.completed =
                        matches!(later_entry.entry_type, EntryType::Task { completed: true });
                }
            }
            _ => {}
        }
    }

    /// Save and exit edit mode (Enter)
    pub fn exit_edit(&mut self) {
        let Some(buffer) = self.edit_buffer.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let content = self.preprocess_content(&buffer.into_content());

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                self.save_daily_edit(entry_index, content);
            }
            InputMode::Edit(EditContext::FilterEdit {
                date, line_index, ..
            }) => {
                self.save_filter_edit(date, line_index, content);
            }
            InputMode::Edit(EditContext::FilterQuickAdd { date, entry_type }) => {
                self.save_filter_quick_add(date, entry_type, content);
            }
            InputMode::Edit(EditContext::LaterEdit {
                source_date,
                line_index,
                ..
            }) => {
                self.save_later_edit(source_date, line_index, content);
            }
            _ => {}
        }
    }

    fn save_daily_edit(&mut self, entry_index: usize, content: String) {
        if content.trim().is_empty() {
            self.delete_at_index_daily(entry_index);
            if let ViewMode::Daily(state) = &mut self.view {
                state.scroll_offset = 0;
            }
        } else if let Some(entry) = self.get_daily_entry_mut(entry_index) {
            entry.content = content;
            self.save();
        }
    }

    fn save_filter_edit(&mut self, date: chrono::NaiveDate, line_index: usize, content: String) {
        if content.trim().is_empty() {
            let _ = storage::delete_entry(date, line_index);
        } else {
            match storage::update_entry_content(date, line_index, content) {
                Ok(false) => {
                    self.set_status(format!(
                        "Failed to update: no entry at index {line_index} for {date}"
                    ));
                }
                Err(e) => {
                    self.set_status(format!("Failed to save: {e}"));
                }
                Ok(true) => {}
            }
        }
        if date == self.current_date {
            let _ = self.reload_current_day();
        }
        let _ = self.refresh_filter();
    }

    fn save_filter_quick_add(
        &mut self,
        date: chrono::NaiveDate,
        entry_type: EntryType,
        content: String,
    ) {
        if !content.trim().is_empty()
            && let Ok(mut lines) = storage::load_day_lines(date)
        {
            let entry = Entry {
                entry_type,
                content,
            };
            lines.push(Line::Entry(entry));
            let _ = storage::save_day_lines(date, &lines);
            if date == self.current_date {
                let _ = self.reload_current_day();
            }
        }
        let _ = self.refresh_filter();
        if let ViewMode::Filter(state) = &mut self.view {
            state.selected = state.entries.len().saturating_sub(1);
        }
    }

    fn save_later_edit(
        &mut self,
        source_date: chrono::NaiveDate,
        line_index: usize,
        content: String,
    ) {
        if content.trim().is_empty() {
            let _ = storage::delete_entry(source_date, line_index);
        } else {
            match storage::update_entry_content(source_date, line_index, content) {
                Ok(false) => {
                    self.set_status(format!(
                        "Failed to update: no entry at index {line_index} for {source_date}"
                    ));
                }
                Err(e) => {
                    self.set_status(format!("Failed to save: {e}"));
                }
                Ok(true) => {}
            }
        }
        if let ViewMode::Daily(state) = &mut self.view {
            state.later_entries =
                storage::collect_later_entries_for_date(self.current_date).unwrap_or_default();
        }
    }

    /// Cancel edit mode without saving (Esc)
    pub fn cancel_edit(&mut self) {
        self.edit_buffer = None;

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                if let Some(entry) = self.get_daily_entry(entry_index)
                    && entry.content.is_empty()
                {
                    self.delete_at_index_daily(entry_index);
                    if let ViewMode::Daily(state) = &mut self.view {
                        state.scroll_offset = 0;
                    }
                }
            }
            InputMode::Edit(EditContext::FilterEdit { .. })
            | InputMode::Edit(EditContext::FilterQuickAdd { .. }) => {}
            InputMode::Edit(EditContext::LaterEdit { .. }) => {}
            _ => {}
        }
    }

    /// Save and add new entry (Tab)
    pub fn commit_and_add_new(&mut self) {
        let Some(buffer) = self.edit_buffer.take() else {
            return;
        };
        let content = buffer.into_content();

        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Edit(EditContext::FilterQuickAdd { date, entry_type }) => {
                self.commit_filter_quick_add(date, entry_type, content);
            }
            InputMode::Edit(EditContext::Daily { entry_index }) => {
                self.commit_daily_entry(entry_index, content);
            }
            _ => {}
        }
    }

    fn commit_filter_quick_add(
        &mut self,
        date: chrono::NaiveDate,
        entry_type: EntryType,
        content: String,
    ) {
        if !content.trim().is_empty()
            && let Ok(mut lines) = storage::load_day_lines(date)
        {
            let line_index = lines.len();
            let entry = Entry {
                entry_type: entry_type.clone(),
                content: content.clone(),
            };
            lines.push(Line::Entry(entry));
            let _ = storage::save_day_lines(date, &lines);
            if date == self.current_date {
                let _ = self.reload_current_day();
            }

            if let ViewMode::Filter(state) = &mut self.view {
                state.entries.push(FilterEntry {
                    source_date: date,
                    line_index,
                    entry_type: entry_type.clone(),
                    content,
                    completed: matches!(entry_type, EntryType::Task { completed: true }),
                });
                state.selected = state.entries.len().saturating_sub(1);
            }
        }
        self.edit_buffer = Some(CursorBuffer::empty());
        self.input_mode = InputMode::Edit(EditContext::FilterQuickAdd {
            date,
            entry_type: match entry_type {
                EntryType::Task { .. } => EntryType::Task { completed: false },
                other => other,
            },
        });
    }

    fn commit_daily_entry(&mut self, entry_index: usize, content: String) {
        if content.trim().is_empty() {
            let was_at_end = entry_index == self.entry_indices.len().saturating_sub(1);
            self.delete_at_index_daily(entry_index);
            if let ViewMode::Daily(state) = &mut self.view
                && !was_at_end
                && state.selected > 0
            {
                state.selected -= 1;
            }
            return;
        }

        let entry_type = self
            .get_daily_entry(entry_index)
            .map(|e| e.entry_type.clone())
            .unwrap_or(EntryType::Task { completed: false });

        if let Some(entry) = self.get_daily_entry_mut(entry_index) {
            entry.content = content;
        }
        self.save();

        let new_entry = Entry {
            entry_type: match entry_type {
                EntryType::Task { .. } => EntryType::Task { completed: false },
                other => other,
            },
            content: String::new(),
        };
        self.add_entry_internal(new_entry, InsertPosition::Below);
    }

    pub(super) fn add_entry_internal(&mut self, entry: Entry, position: InsertPosition) {
        use super::SelectedItem;

        let insert_pos =
            if matches!(position, InsertPosition::Bottom) || self.entry_indices.is_empty() {
                self.lines.len()
            } else {
                match self.get_selected_item() {
                    SelectedItem::Daily { index, .. } => match position {
                        InsertPosition::Below => self.entry_indices[index] + 1,
                        InsertPosition::Above => self.entry_indices[index],
                        InsertPosition::Bottom => unreachable!(),
                    },
                    _ => self.lines.len(),
                }
            };

        self.lines.insert(insert_pos, Line::Entry(entry));
        self.entry_indices = Self::compute_entry_indices(&self.lines);

        let entry_index = self
            .entry_indices
            .iter()
            .position(|&idx| idx == insert_pos)
            .unwrap_or(self.entry_indices.len().saturating_sub(1));

        let visible_index = self.actual_to_visible_index(entry_index);
        if let ViewMode::Daily(state) = &mut self.view {
            state.selected = visible_index;
        }

        self.edit_buffer = Some(CursorBuffer::empty());
        self.input_mode = InputMode::Edit(EditContext::Daily { entry_index });
    }

    pub fn new_task(&mut self, position: InsertPosition) {
        self.add_entry_internal(Entry::new_task(""), position);
    }
}
