//! Tests for the command history system

use tempfile::tempdir;

use crate::history::{CommandHistory, HistoryEntry};

#[test]
fn test_history_basic_functionality() {
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("history_test.json");
    
    // Create history manager
    let history = CommandHistory::with_options(10, &history_file).unwrap();
    
    // Test adding entries
    history.add(
        "test".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    history.add(
        "test2".to_string(),
        vec!["--arg2".to_string(), "value2".to_string()],
        false,
        Some("test error".to_string()),
        None,
    ).unwrap();
    
    // Test getting entries
    let entries = history.get_last(10).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].command, "test2");
    assert_eq!(entries[1].command, "test");
    
    // Test searching
    let search_results = history.search("value1").unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].command, "test");
    
    // Test getting last for command
    let last_test = history.get_last_for_command("test").unwrap();
    assert!(last_test.is_some());
    assert_eq!(last_test.unwrap().command, "test");
    
    // Test clear
    history.clear().unwrap();
    let entries = history.get_last(10).unwrap();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_history_persistence() {
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("persistence_test.json");
    
    // Create history and add entries
    {
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        history.add(
            "test1".to_string(),
            vec!["--arg1".to_string(), "value1".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "test2".to_string(),
            vec!["--arg2".to_string(), "value2".to_string()],
            true,
            None,
            None,
        ).unwrap();
    }
    
    // Create new history instance and verify entries are loaded
    {
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        let entries = history.get_last(10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "test2");
        assert_eq!(entries[1].command, "test1");
    }
}

#[test]
fn test_history_cleanup() {
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("cleanup_test.json");
    
    let history = CommandHistory::with_options(10, &history_file).unwrap();
    
    // Add entry with custom timestamp
    let mut entry1 = HistoryEntry::new(
        "old-cmd".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        true,
        None,
        None,
    );
    entry1.timestamp = 1000; // Old timestamp
    history.add_entry(entry1).unwrap();
    
    let mut entry2 = HistoryEntry::new(
        "new-cmd".to_string(),
        vec!["--arg2".to_string(), "value2".to_string()],
        true,
        None,
        None,
    );
    entry2.timestamp = 2000; // Newer timestamp
    history.add_entry(entry2).unwrap();
    
    // Cleanup entries older than 1500
    let removed = history.cleanup_older_than(1500).unwrap();
    assert_eq!(removed, 1);
    
    // Verify only new entry remains
    let entries = history.get_last(10).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].command, "new-cmd");
}

#[test]
fn test_history_max_size() {
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("max_size_test.json");
    
    // Create history with max size of 2
    let history = CommandHistory::with_options(2, &history_file).unwrap();
    
    // Add 3 entries
    history.add(
        "cmd1".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    history.add(
        "cmd2".to_string(),
        vec!["--arg2".to_string(), "value2".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    history.add(
        "cmd3".to_string(),
        vec!["--arg3".to_string(), "value3".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Verify only 2 entries remain and oldest was removed
    let entries = history.get_last(10).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].command, "cmd3");
    assert_eq!(entries[1].command, "cmd2");
    
    // Verify cmd1 is not in the history
    let cmd1 = history.get_last_for_command("cmd1").unwrap();
    assert!(cmd1.is_none());
}

#[test]
fn test_history_formatted_output() {
    let entry = HistoryEntry::new(
        "test-command".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        true,
        None,
        None,
    );
    
    let formatted = entry.formatted();
    
    // Check the output contains the command name and arguments
    assert!(formatted.contains("test-command"));
    assert!(formatted.contains("--arg1 value1"));
    
    // Check it contains the success indicator
    assert!(formatted.contains("✓"));
    
    // Create a failed entry
    let failed_entry = HistoryEntry::new(
        "failed-command".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        false,
        Some("Error message".to_string()),
        None,
    );
    
    let formatted = failed_entry.formatted();
    
    // Check the output contains the command name and arguments
    assert!(formatted.contains("failed-command"));
    assert!(formatted.contains("--arg1 value1"));
    
    // Check it contains the failure indicator
    assert!(formatted.contains("✗"));
} 