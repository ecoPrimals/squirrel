use crate::ui::layout::LayoutManager;
use super::{Header, Progress};
use std::io::Cursor;

fn strip_ansi(input: &str) -> String {
    String::from_utf8(strip_ansi_escapes::strip(input)).expect("Failed to convert stripped ANSI escapes to UTF-8")
}

#[test]
fn test_header_component() {
    let mut buffer = Cursor::new(Vec::new());
    let mut layout = LayoutManager::new();
    let _ = layout.set_indentation_size(2);
    let header = Header::new("Test Header".to_string());

    header.write(&mut buffer).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let stripped_output = strip_ansi(&output);
    
    // Check header formatting using stripped output
    assert!(stripped_output.contains('╭'));
    assert!(stripped_output.contains('╮'));
    assert!(stripped_output.contains('╰'));
    assert!(stripped_output.contains('╯'));
    assert!(stripped_output.contains('│'));
    assert!(stripped_output.contains("Test Header"));
}

#[test]
fn test_progress_component() {
    let mut buffer = Cursor::new(Vec::new());
    let mut progress = Progress::new(100);

    // Test initial progress
    progress.write(&mut buffer).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("0%"));

    // Test mid progress
    let mut buffer = Cursor::new(Vec::new());
    progress.update_progress(50);
    progress.write(&mut buffer).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("50%"));

    // Test complete progress
    let mut buffer = Cursor::new(Vec::new());
    progress.update_progress(100);
    progress.write(&mut buffer).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("100%"));

    // Test progress clear
    let mut buffer = Cursor::new(Vec::new());
    progress.clear_progress(&mut buffer).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.trim().is_empty());
}

#[test]
fn test_header_with_different_indentation() {
    let mut buffer = Cursor::new(Vec::new());
    let mut layout = LayoutManager::new();
    let _ = layout.set_indentation_size(4);
    let mut header = Header::new("Test Header".to_string());
    header.set_layout(layout);

    header.write(&mut buffer).unwrap();
    let output = String::from_utf8(buffer.into_inner()).unwrap();
    
    // Count spaces before the header
    let first_line = output.lines().next().unwrap();
    let leading_spaces = first_line.chars().take_while(|c| *c == ' ').count();
    assert_eq!(leading_spaces, 0); // Initial indentation is 0
}

#[test]
fn test_progress_with_message() {
    let mut buffer = Cursor::new(Vec::new());
    let mut progress = Progress::new(100);

    // Test progress with message
    progress.print_progress(&mut buffer, "Testing", 0.5).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("Testing"));
    assert!(output.contains("50%"));
}

#[test]
fn test_progress_with_invalid_values() {
    let mut buffer = Cursor::new(Vec::new());
    let mut progress = Progress::new(100);

    // Test negative progress
    progress.print_progress(&mut buffer, "Testing", -0.5).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("0%")); // Should clamp to 0%

    // Test progress > 100%
    let mut buffer = Cursor::new(Vec::new());
    progress.print_progress(&mut buffer, "Testing", 1.5).unwrap();
    let output = strip_ansi(&String::from_utf8(buffer.into_inner()).unwrap());
    assert!(output.contains("100%")); // Should clamp to 100%
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_layout_indentation() {
        let mut layout = LayoutManager::new();
        let _ = layout.set_indentation_size(2);

        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "");

        layout.indent();
        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "  ");
    }

    #[test]
    fn test_layout_nesting() {
        let mut layout = LayoutManager::new();
        let _ = layout.set_indentation_size(2);

        layout.indent();
        layout.indent();
        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "    ");
    }

    #[test]
    fn test_layout_reset() {
        let mut layout = LayoutManager::new();
        let _ = layout.set_indentation_size(4);

        layout.indent();
        layout.indent();
        layout.reset();
        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "");
    }

    #[test]
    fn test_layout_multiple_indents() {
        let mut layout = LayoutManager::new();
        let _ = layout.set_indentation_size(2);

        layout.indent();
        layout.indent();
        layout.dedent().unwrap();
        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "  ");
    }

    #[test]
    fn test_layout_negative_indent() {
        let mut layout = LayoutManager::new();
        let _ = layout.set_indentation_size(2);

        layout.dedent().unwrap_err(); // Should return an error
        let mut buffer = Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "");
    }
} 