// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(warnings)]
// Mock Verification Test - Ensures mocks are only in test code
//
// This test verifies our "zero mocks in production" discipline

#[cfg(test)]
mod mock_verification_tests {
    use std::fs;
    use std::path::Path;

    fn visit_dirs(dir: &Path, violations: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, violations);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    let path_str = path.to_str().unwrap_or("");
                    if path_str.contains("_test")
                        || path_str.contains("/tests/")
                        || path_str.contains("testing")
                        || path_str.contains("mock_providers.rs")
                    {
                        continue;
                    }
                    check_file_for_mocks(&path, violations);
                }
            }
        }
    }

    fn check_file_for_mocks(path: &Path, violations: &mut Vec<String>) {
        if let Ok(content) = fs::read_to_string(path) {
            // If the entire file or large sections are #[cfg(test)], skip detailed check
            if content.contains("struct Mock") || content.contains("fn mock_") {
                // Simple approach: check if mock is within a #[cfg(test)] mod
                let lines: Vec<&str> = content.lines().collect();

                for (i, line) in lines.iter().enumerate() {
                    if (line.contains("struct Mock") || line.contains("fn mock_"))
                        && !line.trim().starts_with("//")
                        && !line.trim().starts_with('*')
                    {
                        // Check if this line is within a #[cfg(test)] module
                        // Look backwards for #[cfg(test)] and mod tests
                        let mut is_in_test_mod = false;
                        for j in (0..i).rev() {
                            if lines[j].contains("#[cfg(test)]") {
                                // Check if there's a mod after this cfg
                                let range_start = (j + 1).min(i);
                                if lines[range_start..=i]
                                    .iter()
                                    .any(|line| line.contains("mod ") && line.contains('{'))
                                {
                                    is_in_test_mod = true;
                                }
                                break;
                            }
                        }

                        if !is_in_test_mod {
                            violations.push(format!(
                                "{}:{} - Found mock in production code: {}",
                                path.display(),
                                i + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn verify_no_production_mocks() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut mock_violations = Vec::new();

        visit_dirs(&src_dir, &mut mock_violations);

        assert!(
            mock_violations.is_empty(),
            "Found {} mock violations in production code:\n{}",
            mock_violations.len(),
            mock_violations.join("\n")
        );
    }

    #[test]
    fn verify_test_mocks_are_isolated() {
        // This test verifies that test helpers are properly marked
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/testing");

        if test_dir.exists() {
            // Verify testing module has proper cfg(test) gates
            let testing_mod = test_dir.join("mod.rs");
            if testing_mod.exists() {
                let content =
                    std::fs::read_to_string(&testing_mod).expect("Failed to read testing/mod.rs");

                // Should have #[cfg(test)] or be clearly marked as test utilities
                assert!(
                    content.contains("#[cfg(test)]")
                        || content.contains("//! Test")
                        || content.contains("//! Mock"),
                    "Testing module should be clearly marked as test-only"
                );
            }
        }
    }
}
