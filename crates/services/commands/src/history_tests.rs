// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_history_entry_creation() {
    let entry = HistoryEntry::new(
        "test-command".to_string(),
        vec!["--arg1".to_string(), "value1".to_string()],
        true,
        None,
        None,
    );

    assert_eq!(entry.command, "test-command");
    assert_eq!(entry.args, vec!["--arg1".to_string(), "value1".to_string()]);
    assert!(entry.success);
    assert!(entry.error_message.is_none());
    assert!(entry.metadata.is_none());
}

#[test]
fn test_history_add_and_get() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("test-history.json");

    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

    // Add some entries
    history
        .add(
            "cmd1".to_string(),
            vec!["arg1".to_string()],
            true,
            None,
            None,
        )
        .expect("should succeed");

    history
        .add(
            "cmd2".to_string(),
            vec!["arg2".to_string()],
            false,
            Some("error".to_string()),
            None,
        )
        .expect("should succeed");

    // Get last entries
    let last_entries = history.get_last(10).expect("should succeed");
    assert_eq!(last_entries.len(), 2);
    assert_eq!(last_entries[0].command, "cmd2");
    assert_eq!(last_entries[1].command, "cmd1");

    // Get last for command
    let last_cmd1 = history
        .get_last_for_command("cmd1")
        .expect("should succeed");
    assert!(last_cmd1.is_some());
    assert_eq!(last_cmd1.expect("should succeed").command, "cmd1");
}

#[test]
fn test_history_search() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("test-history.json");

    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

    // Add some entries
    history
        .add(
            "find".to_string(),
            vec!["file.txt".to_string()],
            true,
            None,
            None,
        )
        .expect("should succeed");

    history
        .add(
            "grep".to_string(),
            vec!["pattern".to_string(), "file.txt".to_string()],
            true,
            None,
            None,
        )
        .expect("should succeed");

    // Search for "file.txt"
    let results = history.search("file.txt").expect("should succeed");
    assert_eq!(results.len(), 2);

    // Search for "pattern"
    let results = history.search("pattern").expect("should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "grep");
}

#[test]
fn test_history_clear() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("test-history.json");

    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

    // Add some entries
    history
        .add(
            "cmd1".to_string(),
            vec!["arg1".to_string()],
            true,
            None,
            None,
        )
        .expect("should succeed");

    history
        .add(
            "cmd2".to_string(),
            vec!["arg2".to_string()],
            true,
            None,
            None,
        )
        .expect("should succeed");

    // Verify entries exist
    let entries = history.get_last(10).expect("should succeed");
    assert_eq!(entries.len(), 2);

    // Clear history
    history.clear().expect("should succeed");

    // Verify entries are gone
    let entries = history.get_last(10).expect("should succeed");
    assert_eq!(entries.len(), 0);

    // Verify file is empty JSON array
    let contents = fs::read_to_string(&history_file).expect("should succeed");
    assert_eq!(contents, "[]");
}

#[test]
fn test_history_cleanup() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("test-history.json");

    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

    // Add entry with custom timestamp
    let mut entry1 = HistoryEntry::new(
        "old-cmd".to_string(),
        vec!["arg1".to_string()],
        true,
        None,
        None,
    );
    entry1.timestamp = 1000; // Old timestamp
    history.add_entry(entry1).expect("should succeed");

    let mut entry2 = HistoryEntry::new(
        "new-cmd".to_string(),
        vec!["arg2".to_string()],
        true,
        None,
        None,
    );
    entry2.timestamp = 2000; // Newer timestamp
    history.add_entry(entry2).expect("should succeed");

    // Cleanup entries older than 1500
    let removed = history.cleanup_older_than(1500).expect("should succeed");
    assert_eq!(removed, 1);

    // Verify only new entry remains
    let entries = history.get_last(10).expect("should succeed");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].command, "new-cmd");
}

#[test]
fn test_history_persistence() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("test-history.json");

    // Create history and add entries
    {
        let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

        history
            .add(
                "cmd1".to_string(),
                vec!["arg1".to_string()],
                true,
                None,
                None,
            )
            .expect("should succeed");

        history
            .add(
                "cmd2".to_string(),
                vec!["arg2".to_string()],
                true,
                None,
                None,
            )
            .expect("should succeed");
    }

    // Create new history instance and verify entries are loaded
    {
        let history = CommandHistory::with_options(10, &history_file).expect("should succeed");

        let entries = history.get_last(10).expect("should succeed");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "cmd2");
        assert_eq!(entries[1].command, "cmd1");
    }
}

#[test]
fn test_get_last_zero_returns_empty() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("h.json");
    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");
    history
        .add("a".to_string(), vec![], true, None, None)
        .expect("should succeed");
    assert!(history.get_last(0).expect("should succeed").is_empty());
}

#[test]
fn test_search_no_matches_returns_empty() {
    let dir = tempdir().expect("should succeed");
    let history_file = dir.path().join("h.json");
    let history = CommandHistory::with_options(10, &history_file).expect("should succeed");
    history
        .add("ls".to_string(), vec![], true, None, None)
        .expect("should succeed");
    assert!(
        history
            .search("zzznomatch")
            .expect("should succeed")
            .is_empty()
    );
}

#[test]
fn test_history_entry_formatted_contains_command() {
    let e = HistoryEntry::new("echo".to_string(), vec!["hi".to_string()], true, None, None);
    let s = e.formatted();
    assert!(s.contains("echo"));
    assert!(s.contains("hi"));
}

#[test]
fn test_load_ndjson_fallback_when_array_invalid() {
    let dir = tempdir().expect("should succeed");
    let path = dir.path().join("mixed.json");
    let line = serde_json::to_string(&HistoryEntry::new(
        "c1".to_string(),
        vec![],
        true,
        None,
        None,
    ))
    .expect("serialize entry");
    fs::write(&path, format!("not-a-json-array\n{line}\n")).expect("should succeed");
    let history = CommandHistory::with_options(10, &path).expect("should succeed");
    let entries = history.get_last(10).expect("should succeed");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].command, "c1");
    let ts = entries[0].timestamp;
    let removed = history.cleanup_older_than(ts + 1).expect("should succeed");
    assert_eq!(removed, 1);
}

#[test]
fn test_add_respects_max_size_in_memory() {
    let dir = tempdir().expect("should succeed");
    let path = dir.path().join("cap.json");
    let history = CommandHistory::with_options(3, &path).expect("should succeed");
    for i in 0..10 {
        history
            .add(format!("cmd{i}"), vec![], true, None, None)
            .expect("should succeed");
    }
    assert_eq!(history.get_last(100).expect("should succeed").len(), 3);
}
