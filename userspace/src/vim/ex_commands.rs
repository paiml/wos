// Ex command parsing and execution (:w, :q, :wq, etc.)

use super::state::{VimError, VimState};

/// Parse and execute an ex command (command that starts with :)
pub fn execute_ex_command(state: &mut VimState, command: &str) -> Result<String, VimError> {
    let cmd = command.trim();

    match cmd {
        "w" | "write" => {
            // Mark buffer as saved - actual VFS write happens at integration layer
            let file_path = state.current_buffer().file_path.clone();
            state.current_buffer_mut().mark_saved();
            state.update_modified();
            Ok(format!("\"{}\" written", file_path.display()))
        }

        "q" | "quit" => {
            if state.modified {
                Err(VimError::General(
                    "No write since last change (add ! to override)".to_string(),
                ))
            } else {
                // Return success - application exit is handled at integration layer
                Ok("Quitting...".to_string())
            }
        }

        "q!" | "quit!" => {
            // Force quit without saving
            Ok("Quitting...".to_string())
        }

        "wq" | "x" => {
            // Write and quit
            execute_ex_command(state, "w")?;
            execute_ex_command(state, "q")
        }

        "help" => Ok(
            "Vim commands: h/j/k/l (move), i (insert), x (delete), u (undo), :w (save), :q (quit)"
                .to_string(),
        ),

        "" => Ok(String::new()),

        _ => Err(VimError::InvalidCommand(cmd.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_command() {
        let mut state = VimState::new();
        state.current_buffer_mut().mark_modified();
        state.update_modified();

        let result = execute_ex_command(&mut state, "w");
        assert!(result.is_ok());
        assert!(!state.modified);
    }

    #[test]
    fn test_quit_with_unsaved_changes() {
        let mut state = VimState::new();
        state.current_buffer_mut().mark_modified();
        state.update_modified();

        let result = execute_ex_command(&mut state, "q");
        assert!(result.is_err());
    }

    #[test]
    fn test_quit_force() {
        let mut state = VimState::new();
        state.current_buffer_mut().mark_modified();
        state.update_modified();

        let result = execute_ex_command(&mut state, "q!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_help_command() {
        let mut state = VimState::new();

        let result = execute_ex_command(&mut state, "help");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Vim commands"));
    }

    #[test]
    fn test_wq_command() {
        let mut state = VimState::new();
        state.current_buffer_mut().mark_modified();
        state.update_modified();

        let result = execute_ex_command(&mut state, "wq");
        assert!(result.is_ok());
        assert!(!state.modified);
    }

    #[test]
    fn test_x_command() {
        let mut state = VimState::new();
        state.current_buffer_mut().mark_modified();
        state.update_modified();

        let result = execute_ex_command(&mut state, "x");
        assert!(result.is_ok());
        assert!(!state.modified);
    }

    #[test]
    fn test_empty_command() {
        let mut state = VimState::new();

        let result = execute_ex_command(&mut state, "");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_invalid_command() {
        let mut state = VimState::new();

        let result = execute_ex_command(&mut state, "invalid");
        assert!(result.is_err());
    }
}
