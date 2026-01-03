use std::io;

use chrono::{Local, NaiveDate};

use crate::cursor::CursorBuffer;
use crate::storage::{self, EntryType};

use super::{
    App, DailyState, EditContext, FILTER_HEADER_LINES, FilterState, InputMode, SelectedItem,
    ViewMode,
};

impl App {
    pub fn enter_filter_input(&mut self) {
        match &mut self.view {
            ViewMode::Filter(state) => {
                state.query_buffer.set_content(&state.query);
            }
            ViewMode::Daily(_) => {
                self.command_buffer.clear();
            }
        }

        self.input_mode = InputMode::QueryInput;
    }

    /// Shared helper for applying a filter query and switching to filter view.
    fn apply_filter(&mut self, query: String) -> io::Result<()> {
        let (query, unknown_filters) = storage::expand_saved_filters(&query, &self.config.filters);
        let mut filter = storage::parse_filter_query(&query);
        filter.invalid_tokens.extend(unknown_filters);

        if !filter.invalid_tokens.is_empty() {
            self.set_status(format!(
                "Unknown filter: {}",
                filter.invalid_tokens.join(", ")
            ));
        }

        let entries = storage::collect_filtered_entries(&filter)?;
        let selected = entries.len().saturating_sub(1);

        self.view = ViewMode::Filter(FilterState {
            query,
            query_buffer: CursorBuffer::empty(),
            entries,
            selected,
            scroll_offset: 0,
        });
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    /// Extracts the query from the appropriate buffer based on current view.
    fn extract_query_buffer(&mut self) -> String {
        match &mut self.view {
            ViewMode::Filter(state) => {
                let q = state.query_buffer.content().to_string();
                state.query_buffer.clear();
                q
            }
            ViewMode::Daily(_) => {
                let q = self.command_buffer.content().to_string();
                self.command_buffer.clear();
                q
            }
        }
    }

    pub fn execute_filter(&mut self) -> io::Result<()> {
        self.save();
        let query = self.extract_query_buffer();
        self.apply_filter(query)
    }

    pub fn quick_filter(&mut self, query: &str) -> io::Result<()> {
        self.save();
        self.apply_filter(query.to_string())
    }

    pub fn cancel_filter_input(&mut self) {
        match &mut self.view {
            ViewMode::Filter(state) => {
                state.query_buffer.clear();
            }
            ViewMode::Daily(_) => {
                self.command_buffer.clear();
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn exit_filter(&mut self) {
        if let ViewMode::Filter(state) = &self.view {
            self.last_filter_query = Some(state.query.clone());
        }
        let later_entries =
            storage::collect_later_entries_for_date(self.current_date).unwrap_or_default();
        self.view = ViewMode::Daily(DailyState::new(self.entry_indices.len(), later_entries));
        self.input_mode = InputMode::Normal;
    }

    pub fn return_to_filter(&mut self) -> io::Result<()> {
        let query = self
            .last_filter_query
            .clone()
            .unwrap_or_else(|| self.config.default_filter.clone());
        self.quick_filter(&query)
    }

    pub fn refresh_filter(&mut self) -> io::Result<()> {
        let ViewMode::Filter(state) = &mut self.view else {
            return Ok(());
        };

        let filter = storage::parse_filter_query(&state.query);

        if !filter.invalid_tokens.is_empty() {
            self.status_message = Some(format!(
                "Unknown filter: {}",
                filter.invalid_tokens.join(", ")
            ));
        }

        state.entries = storage::collect_filtered_entries(&filter)?;
        state.selected = state.selected.min(state.entries.len().saturating_sub(1));
        state.scroll_offset = 0;
        Ok(())
    }

    /// Navigate to a specific day and select the entry at the given line index
    fn goto_entry_source(&mut self, date: NaiveDate, line_index: usize) -> io::Result<()> {
        if let ViewMode::Filter(state) = &self.view {
            self.last_filter_query = Some(state.query.clone());
        }
        if date != self.current_date {
            self.save();
        }

        let later_entries = self.load_day(date)?;
        let later_count = later_entries.len();

        let entry_pos = self
            .entry_indices
            .iter()
            .position(|&i| i == line_index)
            .unwrap_or(0);
        let selected = later_count + entry_pos;

        self.view = ViewMode::Daily(DailyState {
            selected,
            scroll_offset: 0,
            original_lines: None,
            later_entries,
        });
        self.input_mode = InputMode::Normal;
        self.edit_buffer = None;

        Ok(())
    }

    /// View the source day of the currently selected entry (unified across views)
    pub fn view_entry_source(&mut self) -> io::Result<()> {
        match self.get_selected_item() {
            SelectedItem::Filter { entry, .. } => {
                self.goto_entry_source(entry.source_date, entry.line_index)
            }
            SelectedItem::Later { entry, .. } => {
                self.goto_entry_source(entry.source_date, entry.line_index)
            }
            SelectedItem::Daily { .. } | SelectedItem::None => Ok(()),
        }
    }

    pub fn filter_quick_add(&mut self) {
        let today = Local::now().date_naive();
        self.edit_buffer = Some(CursorBuffer::empty());
        self.input_mode = InputMode::Edit(EditContext::FilterQuickAdd {
            date: today,
            entry_type: EntryType::Task { completed: false },
        });
    }

    #[must_use]
    pub fn filter_visual_line(&self) -> usize {
        let ViewMode::Filter(state) = &self.view else {
            return 0;
        };
        state.selected + FILTER_HEADER_LINES
    }

    #[must_use]
    pub fn filter_total_lines(&self) -> usize {
        let ViewMode::Filter(state) = &self.view else {
            return 1;
        };
        state.entries.len() + FILTER_HEADER_LINES
    }
}
