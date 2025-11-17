// Macro recording and playback tests (TDD RED phase)
// Tests for vim macro functionality: q{register}, @{register}, @@

#[cfg(test)]
mod tests {
    use crate::vim::state::{ParserState, Register, VimState};

    #[test]
    fn test_macro_recording_start() {
        let mut state = VimState::new();

        // Start recording to register 'a'
        state.start_macro_recording(Register::Named('a'));

        assert!(state.is_recording_macro());
        assert_eq!(state.recording_register(), Some(&Register::Named('a')));
    }

    #[test]
    fn test_macro_recording_stop() {
        let mut state = VimState::new();

        // Start recording
        state.start_macro_recording(Register::Named('a'));
        assert!(state.is_recording_macro());

        // Stop recording
        state.stop_macro_recording();
        assert!(!state.is_recording_macro());
    }

    #[test]
    fn test_macro_record_keystrokes() {
        let mut state = VimState::new();

        // Start recording to register 'a'
        state.start_macro_recording(Register::Named('a'));

        // Record some keystrokes
        state.record_keystroke('i'); // Insert mode
        state.record_keystroke('h'); // Type 'h'
        state.record_keystroke('i'); // Type 'i'
        state.record_keystroke('\x1b'); // ESC to normal mode

        // Stop recording
        state.stop_macro_recording();

        // Verify macro was saved to register
        let macro_content = state.get_macro(&Register::Named('a'));
        assert!(macro_content.is_some());
        assert_eq!(macro_content.unwrap(), "ihi\x1b");
    }

    #[test]
    fn test_macro_playback() {
        let mut state = VimState::new();

        // Manually set up a macro in register 'a'
        state.set_macro(Register::Named('a'), "itest\x1b".to_string());

        // Get the macro for playback
        let macro_str = state.get_macro(&Register::Named('a')).unwrap();
        assert_eq!(macro_str, "itest\x1b");
    }

    #[test]
    fn test_macro_last_executed() {
        let mut state = VimState::new();

        // Set up two macros
        state.set_macro(Register::Named('a'), "ia\x1b".to_string());
        state.set_macro(Register::Named('b'), "ib\x1b".to_string());

        // Mark 'a' as last executed
        state.set_last_executed_macro(Register::Named('a'));
        assert_eq!(state.last_executed_macro(), Some(&Register::Named('a')));

        // Mark 'b' as last executed
        state.set_last_executed_macro(Register::Named('b'));
        assert_eq!(state.last_executed_macro(), Some(&Register::Named('b')));
    }

    #[test]
    fn test_macro_recording_ignores_q_command() {
        let mut state = VimState::new();

        // Start recording to register 'a'
        state.start_macro_recording(Register::Named('a'));

        // Record keystrokes including 'q' (which should be recorded as normal keystroke)
        state.record_keystroke('q');
        state.record_keystroke('w');

        // Stop recording with explicit call (in real impl, 'q' in normal mode stops)
        state.stop_macro_recording();

        // The 'q' before stop should be recorded
        let macro_content = state.get_macro(&Register::Named('a')).unwrap();
        assert_eq!(macro_content, "qw");
    }

    #[test]
    fn test_macro_empty_recording() {
        let mut state = VimState::new();

        // Start and immediately stop recording
        state.start_macro_recording(Register::Named('a'));
        state.stop_macro_recording();

        // Empty macro should be saved as empty string
        let macro_content = state.get_macro(&Register::Named('a'));
        assert!(macro_content.is_some());
        assert_eq!(macro_content.unwrap(), "");
    }

    #[test]
    fn test_macro_overwrite_existing() {
        let mut state = VimState::new();

        // Record first macro to register 'a'
        state.start_macro_recording(Register::Named('a'));
        state.record_keystroke('i');
        state.record_keystroke('1');
        state.record_keystroke('\x1b');
        state.stop_macro_recording();

        let first_macro = state.get_macro(&Register::Named('a')).unwrap();
        assert_eq!(first_macro, "i1\x1b");

        // Record second macro to same register (overwrite)
        state.start_macro_recording(Register::Named('a'));
        state.record_keystroke('i');
        state.record_keystroke('2');
        state.record_keystroke('\x1b');
        state.stop_macro_recording();

        // Should be overwritten
        let second_macro = state.get_macro(&Register::Named('a')).unwrap();
        assert_eq!(second_macro, "i2\x1b");
    }

    #[test]
    fn test_macro_recording_multiple_registers() {
        let mut state = VimState::new();

        // Record to register 'a'
        state.start_macro_recording(Register::Named('a'));
        state.record_keystroke('i');
        state.record_keystroke('a');
        state.record_keystroke('\x1b');
        state.stop_macro_recording();

        // Record to register 'b'
        state.start_macro_recording(Register::Named('b'));
        state.record_keystroke('i');
        state.record_keystroke('b');
        state.record_keystroke('\x1b');
        state.stop_macro_recording();

        // Both should exist independently
        assert_eq!(state.get_macro(&Register::Named('a')).unwrap(), "ia\x1b");
        assert_eq!(state.get_macro(&Register::Named('b')).unwrap(), "ib\x1b");
    }

    #[test]
    fn test_macro_recording_with_motion_commands() {
        let mut state = VimState::new();

        // Record macro with motion commands
        state.start_macro_recording(Register::Named('m'));
        state.record_keystroke('w'); // Move forward word
        state.record_keystroke('d'); // Delete
        state.record_keystroke('w'); // Delete word
        state.stop_macro_recording();

        let macro_content = state.get_macro(&Register::Named('m')).unwrap();
        assert_eq!(macro_content, "wdw");
    }

    #[test]
    fn test_macro_not_recording_when_stopped() {
        let mut state = VimState::new();

        // Record, then stop
        state.start_macro_recording(Register::Named('a'));
        state.record_keystroke('i');
        state.stop_macro_recording();

        // Try to record more (should not be added)
        state.record_keystroke('x');
        state.record_keystroke('y');

        // Should only have the first keystroke
        assert_eq!(state.get_macro(&Register::Named('a')).unwrap(), "i");
    }

    // ============================================================================
    // Command Integration Tests (q, @, @@)
    // ============================================================================

    #[test]
    fn test_q_command_starts_recording() {
        let mut state = VimState::new();

        // Press 'q' in normal mode
        let result = state.handle_normal_key('q');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMacroRegister);
        assert!(!state.is_recording_macro());

        // Press 'a' to select register
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);
        assert!(state.is_recording_macro());
        assert_eq!(state.recording_register(), Some(&Register::Named('a')));
    }

    #[test]
    fn test_q_command_stops_recording() {
        let mut state = VimState::new();

        // Start recording to register 'a'
        state.start_macro_recording(Register::Named('a'));
        state.record_keystroke('i');
        state.record_keystroke('t');
        state.record_keystroke('e');
        state.record_keystroke('s');
        state.record_keystroke('t');
        state.record_keystroke('\x1b');

        // Press 'q' while recording to stop
        assert!(state.is_recording_macro());
        let result = state.handle_normal_key('q');
        assert!(result.is_ok());
        assert!(!state.is_recording_macro());

        // Verify macro was saved
        assert_eq!(state.get_macro(&Register::Named('a')).unwrap(), "itest\x1b");
    }

    #[test]
    fn test_q_command_invalid_register() {
        let mut state = VimState::new();

        // Press 'q' then invalid character
        state.handle_normal_key('q').unwrap();
        let result = state.handle_normal_key('1'); // Numbers not allowed
        assert!(result.is_err());
        assert_eq!(state.parser_state, ParserState::Normal);
        assert!(!state.is_recording_macro());
    }

    #[test]
    fn test_at_command_replays_macro() {
        let mut state = VimState::new();

        // Set up a macro in register 'a'
        state.set_macro(Register::Named('a'), "itest\x1b".to_string());

        // Press '@' in normal mode
        let result = state.handle_normal_key('@');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::AwaitingMacroPlayback);

        // Press 'a' to replay macro from register 'a'
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());
        assert_eq!(state.parser_state, ParserState::Normal);
        assert_eq!(state.last_executed_macro(), Some(&Register::Named('a')));
    }

    #[test]
    fn test_at_command_no_macro_error() {
        let mut state = VimState::new();

        // Press '@' then 'b' (no macro in register 'b')
        state.handle_normal_key('@').unwrap();
        let result = state.handle_normal_key('b');
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No macro recorded"));
    }

    #[test]
    fn test_double_at_replays_last_macro() {
        let mut state = VimState::new();

        // Set up macros in two registers
        state.set_macro(Register::Named('a'), "ia\x1b".to_string());
        state.set_macro(Register::Named('b'), "ib\x1b".to_string());

        // Execute macro 'a'
        state.handle_normal_key('@').unwrap();
        state.handle_normal_key('a').unwrap();

        // Now execute @@
        state.handle_normal_key('@').unwrap();
        let result = state.handle_normal_key('@');
        assert!(result.is_ok());
        assert_eq!(state.last_executed_macro(), Some(&Register::Named('a')));
    }

    #[test]
    fn test_double_at_no_previous_macro() {
        let mut state = VimState::new();

        // Try @@ without having executed any macro
        state.handle_normal_key('@').unwrap();
        let result = state.handle_normal_key('@');
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No previous macro"));
    }

    #[test]
    fn test_macro_workflow_end_to_end() {
        let mut state = VimState::new();

        // 1. Start recording with qa
        state.handle_normal_key('q').unwrap();
        state.handle_normal_key('a').unwrap();
        assert!(state.is_recording_macro());

        // 2. Record some keystrokes
        state.record_keystroke('i');
        state.record_keystroke('h');
        state.record_keystroke('e');
        state.record_keystroke('l');
        state.record_keystroke('l');
        state.record_keystroke('o');
        state.record_keystroke('\x1b');

        // 3. Stop recording with q
        state.handle_normal_key('q').unwrap();
        assert!(!state.is_recording_macro());

        // 4. Verify macro was saved
        assert_eq!(
            state.get_macro(&Register::Named('a')).unwrap(),
            "ihello\x1b"
        );

        // 5. Replay with @a
        state.handle_normal_key('@').unwrap();
        let result = state.handle_normal_key('a');
        assert!(result.is_ok());

        // 6. Replay again with @@
        state.handle_normal_key('@').unwrap();
        let result = state.handle_normal_key('@');
        assert!(result.is_ok());
    }
}
