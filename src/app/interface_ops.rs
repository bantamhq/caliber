use std::io;

use super::{App, InputMode, InterfaceContext, ProjectInterfaceState, TagInterfaceState};

/// Visible height for list interfaces (popup height minus borders and query line)
const LIST_VISIBLE_HEIGHT: usize = 8;

/// Common behavior for list-based interfaces (Project, Tag)
trait ListInterface {
    fn input_focused(&self) -> bool;
    fn set_input_focused(&mut self, focused: bool);
    fn list_len(&self) -> usize;
    fn selected(&self) -> usize;
    fn set_selected(&mut self, idx: usize);
    fn scroll_offset(&self) -> usize;
    fn set_scroll_offset(&mut self, offset: usize);
}

impl ListInterface for ProjectInterfaceState {
    fn input_focused(&self) -> bool {
        self.input_focused
    }
    fn set_input_focused(&mut self, focused: bool) {
        self.input_focused = focused;
    }
    fn list_len(&self) -> usize {
        self.filtered_indices.len()
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn set_selected(&mut self, idx: usize) {
        self.selected = idx;
    }
    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }
}

impl ListInterface for TagInterfaceState {
    fn input_focused(&self) -> bool {
        self.input_focused
    }
    fn set_input_focused(&mut self, focused: bool) {
        self.input_focused = focused;
    }
    fn list_len(&self) -> usize {
        self.filtered_indices.len()
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn set_selected(&mut self, idx: usize) {
        self.selected = idx;
    }
    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }
}

/// Move selection up in a list interface, adjusting scroll if needed
fn list_move_up(state: &mut impl ListInterface) {
    if state.selected() > 0 {
        state.set_selected(state.selected() - 1);
        if state.selected() < state.scroll_offset() {
            state.set_scroll_offset(state.selected());
        }
    }
}

/// Move selection down in a list interface, adjusting scroll if needed
fn list_move_down(state: &mut impl ListInterface) {
    if state.selected() + 1 < state.list_len() {
        state.set_selected(state.selected() + 1);
        if state.selected() >= state.scroll_offset() + LIST_VISIBLE_HEIGHT {
            state.set_scroll_offset(state.selected() - LIST_VISIBLE_HEIGHT + 1);
        }
    }
}

impl App {
    /// Toggle focus between list/calendar and input field in any interface
    pub fn interface_toggle_focus(&mut self) {
        match &mut self.input_mode {
            InputMode::Interface(InterfaceContext::Date(state)) => {
                state.input_focused = !state.input_focused;
            }
            InputMode::Interface(InterfaceContext::Project(state)) => {
                state.set_input_focused(!state.input_focused());
            }
            InputMode::Interface(InterfaceContext::Tag(state)) => {
                state.set_input_focused(!state.input_focused());
            }
            _ => {}
        }
    }

    /// Check if input is focused in current interface
    pub fn interface_input_focused(&self) -> bool {
        match &self.input_mode {
            InputMode::Interface(InterfaceContext::Date(state)) => state.input_focused,
            InputMode::Interface(InterfaceContext::Project(state)) => state.input_focused(),
            InputMode::Interface(InterfaceContext::Tag(state)) => state.input_focused(),
            _ => false,
        }
    }

    /// Unified move up for all interfaces (when list/calendar focused)
    pub fn interface_move_up(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        match &mut self.input_mode {
            InputMode::Interface(InterfaceContext::Date(_)) => {
                self.date_interface_move(0, -1);
            }
            InputMode::Interface(InterfaceContext::Project(state)) => {
                list_move_up(state);
            }
            InputMode::Interface(InterfaceContext::Tag(state)) => {
                list_move_up(state);
            }
            _ => {}
        }
    }

    /// Unified move down for all interfaces (when list/calendar focused)
    pub fn interface_move_down(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        match &mut self.input_mode {
            InputMode::Interface(InterfaceContext::Date(_)) => {
                self.date_interface_move(0, 1);
            }
            InputMode::Interface(InterfaceContext::Project(state)) => {
                list_move_down(state);
            }
            InputMode::Interface(InterfaceContext::Tag(state)) => {
                list_move_down(state);
            }
            _ => {}
        }
    }

    /// Unified move left (date interface only, when calendar focused)
    pub fn interface_move_left(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        if matches!(
            self.input_mode,
            InputMode::Interface(InterfaceContext::Date(_))
        ) {
            self.date_interface_move(-1, 0);
        }
    }

    /// Unified move right (date interface only, when calendar focused)
    pub fn interface_move_right(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        if matches!(
            self.input_mode,
            InputMode::Interface(InterfaceContext::Date(_))
        ) {
            self.date_interface_move(1, 0);
        }
    }

    /// Unified submit/select action (context-aware)
    pub fn interface_submit(&mut self) -> io::Result<()> {
        match &self.input_mode {
            InputMode::Interface(InterfaceContext::Date(_)) => {
                if self.interface_input_focused() {
                    self.date_interface_submit_input()?;
                } else {
                    self.confirm_date_interface()?;
                }
            }
            InputMode::Interface(InterfaceContext::Project(_)) => {
                if self.interface_input_focused() {
                    self.project_interface_autocomplete_submit()?;
                } else {
                    self.project_interface_select()?;
                }
            }
            InputMode::Interface(InterfaceContext::Tag(_)) => {
                if self.interface_input_focused() {
                    self.tag_interface_autocomplete_submit()?;
                } else {
                    self.tag_interface_select()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Delete/remove action (project/tag only, when list focused)
    pub fn interface_delete(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        match &mut self.input_mode {
            InputMode::Interface(InterfaceContext::Project(state)) => {
                if let Some(id) = state.remove_selected() {
                    self.set_status(format!("Removed {id} from registry"));
                }
            }
            InputMode::Interface(InterfaceContext::Tag(_)) => {
                self.tag_interface_delete();
            }
            _ => {}
        }
    }

    /// Rename action (tag only, when list focused)
    pub fn interface_rename(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        if matches!(
            self.input_mode,
            InputMode::Interface(InterfaceContext::Tag(_))
        ) {
            self.tag_interface_rename();
        }
    }

    /// Hide action (project only, when list focused)
    pub fn interface_hide(&mut self) {
        if self.interface_input_focused() {
            return;
        }

        if let InputMode::Interface(InterfaceContext::Project(ref mut state)) = self.input_mode
            && let Some(id) = state.hide_selected()
        {
            self.set_status(format!("Hidden {id} from registry"));
        }
    }

    /// Autocomplete submit for project interface
    fn project_interface_autocomplete_submit(&mut self) -> io::Result<()> {
        let InputMode::Interface(InterfaceContext::Project(ref state)) = self.input_mode else {
            return Ok(());
        };

        if !state.filtered_indices.is_empty() {
            let first_match = state
                .selected_project()
                .map(|p| (p.id.clone(), p.available));

            if let Some((id, true)) = first_match {
                self.input_mode = InputMode::Normal;
                self.open_journal(&id)?;
            } else {
                self.set_status("Project not available");
            }
        } else {
            self.set_status("No matching projects");
        }

        Ok(())
    }

    /// Autocomplete submit for tag interface
    fn tag_interface_autocomplete_submit(&mut self) -> io::Result<()> {
        let InputMode::Interface(InterfaceContext::Tag(ref state)) = self.input_mode else {
            return Ok(());
        };

        if let Some(tag) = state.selected_tag() {
            let tag_name = tag.name.clone();
            self.input_mode = InputMode::Normal;
            self.quick_filter(&format!("#{}", tag_name))?;
        } else {
            self.set_status("No matching tags");
        }

        Ok(())
    }
}
