use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ui_terminal::widgets::chat::{ChatState, InputMode};

#[test]
fn test_input_mode_handling() {
    // Create a chat state
    let mut state = ChatState::new();
    
    // By default, we should be in Normal mode
    assert_eq!(state.input_mode, InputMode::Normal);
    
    // In normal mode, 'q' should trigger quit
    let (handled, should_quit) = state.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
    assert!(handled);
    assert!(should_quit);
    
    // In normal mode, 'i' should switch to edit mode
    let (handled, should_quit) = state.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    assert!(handled);
    assert!(!should_quit);
    assert_eq!(state.input_mode, InputMode::Editing);
    
    // In edit mode, 'q' should NOT trigger quit, but be inserted as text
    let (handled, should_quit) = state.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
    assert!(handled);
    assert!(!should_quit);
    assert_eq!(state.input, "q"); // The 'q' should be added to input
    
    // In edit mode, Esc should return to normal mode
    let (handled, should_quit) = state.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert!(handled);
    assert!(!should_quit);
    assert_eq!(state.input_mode, InputMode::Normal);
    
    // Back to edit mode to test more input
    state.enter_edit_mode();
    
    // Add some text and verify it's inserted
    state.handle_key_event(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()));
    state.handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
    state.handle_key_event(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    state.handle_key_event(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    state.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty()));
    
    assert_eq!(state.input, "qhello");
    
    // Backspace should remove a character
    state.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
    assert_eq!(state.input, "qhell");
    
    // Enter should send the message and clear input
    state.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(state.input, "");
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].content, "qhell");
    assert!(state.messages[0].is_user);
}

#[test]
fn test_input_mode_enum_helper() {
    // Test the InputMode helper method that determines if a key should be processed globally
    
    // In normal mode, all keys should be processed globally
    assert!(InputMode::Normal.should_process_globally(KeyCode::Char('q')));
    assert!(InputMode::Normal.should_process_globally(KeyCode::Char('i')));
    assert!(InputMode::Normal.should_process_globally(KeyCode::Enter));
    
    // In editing mode, only specific keys should be processed globally
    assert!(!InputMode::Editing.should_process_globally(KeyCode::Char('q'))); // Not global - should be inserted
    assert!(!InputMode::Editing.should_process_globally(KeyCode::Char('a'))); // Not global - should be inserted
    
    // But control keys should still be processed globally
    assert!(InputMode::Editing.should_process_globally(KeyCode::Esc));
    assert!(InputMode::Editing.should_process_globally(KeyCode::Enter));
} 