use std::io;

use crate::storage;

use super::{App, ConfirmContext, InputMode};

impl App {
    pub fn execute_command(&mut self) -> io::Result<()> {
        let cmd = self.command_buffer.content().to_string();
        self.command_buffer.clear();
        let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
        let command = parts.first().copied().unwrap_or("");
        let arg = parts.get(1).copied().unwrap_or("").trim();

        match command {
            "q" | "quit" => {
                self.save();
                self.should_quit = true;
            }
            "goto" | "g" => {
                if arg.is_empty() {
                    self.set_status("Usage: :goto YYYY/MM/DD or :goto MM/DD");
                } else if let Some(date) = Self::parse_goto_date(arg) {
                    self.goto_day(date)?;
                } else {
                    self.set_status(format!("Invalid date: {arg}"));
                }
            }
            "o" | "open" => {
                if arg.is_empty() {
                    self.set_status("Usage: :open /path/to/file.md");
                } else {
                    self.open_journal(arg)?;
                }
            }
            "config-reload" => {
                self.reload_config()?;
            }
            "global" => {
                self.switch_to_global()?;
            }
            "project" => {
                if arg.is_empty() {
                    if storage::get_project_path().is_some() {
                        self.switch_to_project()?;
                    } else if self.in_git_repo {
                        self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                        return Ok(());
                    } else {
                        self.set_status("Not in a git repository - no project journal available");
                    }
                } else {
                    self.open_journal(arg)?;
                }
            }
            "init-project" => {
                if storage::get_project_path().is_some() {
                    self.set_status("Project journal already exists");
                } else if self.in_git_repo {
                    self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                    return Ok(());
                } else {
                    let cwd = std::env::current_dir()?;
                    let caliber_dir = cwd.join(".caliber");
                    std::fs::create_dir_all(&caliber_dir)?;
                    let journal_path = caliber_dir.join("journal.md");
                    if !journal_path.exists() {
                        std::fs::write(&journal_path, "")?;
                    }
                    storage::set_project_path(journal_path);
                    self.switch_to_project()?;
                    self.set_status("Project journal created");
                }
            }
            _ => {}
        }
        self.input_mode = InputMode::Normal;
        Ok(())
    }
}
