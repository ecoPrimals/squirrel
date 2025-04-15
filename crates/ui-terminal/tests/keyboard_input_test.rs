use ui_terminal::widgets::chat::InputMode;
use crossterm::event::KeyCode;
use ui_terminal::should_process_as_global_command;

/// This test confirms that 'q' key is processed differently based on input mode
#[test]
fn test_q_key_processing() {
    // In Normal mode, 'q' should be processed as a global command (quit)
    assert!(should_process_as_global_command(&InputMode::Normal, KeyCode::Char('q')));
    
    // In Editing mode, 'q' should NOT be processed as a global command
    // It should be added to the text input instead
    assert!(!should_process_as_global_command(&InputMode::Editing, KeyCode::Char('q')));
}

/// Test that Enter and Esc are always processed as global commands regardless of mode
#[test]
fn test_control_keys_always_global() {
    // Test Enter key
    assert!(should_process_as_global_command(&InputMode::Normal, KeyCode::Enter));
    assert!(should_process_as_global_command(&InputMode::Editing, KeyCode::Enter));
    
    // Test Esc key
    assert!(should_process_as_global_command(&InputMode::Normal, KeyCode::Esc));
    assert!(should_process_as_global_command(&InputMode::Editing, KeyCode::Esc));
}

/// Test that regular character keys are not global commands in Editing mode
#[test]
fn test_character_keys_not_global_in_edit_mode() {
    // These should all be false in editing mode
    assert!(!should_process_as_global_command(&InputMode::Editing, KeyCode::Char('a')));
    assert!(!should_process_as_global_command(&InputMode::Editing, KeyCode::Char('q')));
    assert!(!should_process_as_global_command(&InputMode::Editing, KeyCode::Char('1')));
    assert!(!should_process_as_global_command(&InputMode::Editing, KeyCode::Char(' ')));
    
    // But true in normal mode
    assert!(should_process_as_global_command(&InputMode::Normal, KeyCode::Char('a')));
    assert!(should_process_as_global_command(&InputMode::Normal, KeyCode::Char('q')));
} 