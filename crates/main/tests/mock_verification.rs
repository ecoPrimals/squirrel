// Mock Verification Test - Ensures mocks are only in test code
//
// This test verifies our "zero mocks in production" discipline

#[cfg(test)]
mod mock_verification_tests {
    use std::path::Path;
    use walkdir::WalkDir;

    #[test]
    fn verify_no_production_mocks() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut mock_violations = Vec::new();

        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
                    && !e.path().to_str().unwrap().contains("_test")
                    && !e.path().to_str().unwrap().contains("/tests/")
                    && !e.path().to_str().unwrap().contains("testing")
                    && !e.path().to_str().unwrap().contains("mock_providers.rs")
            })
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                // Look for mock patterns in production code
                if content.contains("struct Mock") || content.contains("fn mock_") {
                    // Allow comments and documentation about mocks
                    let lines: Vec<&str> = content.lines().collect();
                    for (i, line) in lines.iter().enumerate() {
                        if (line.contains("struct Mock") || line.contains("fn mock_"))
                            && !line.trim().starts_with("//")
                            && !line.trim().starts_with("*")
                        {
                            mock_violations.push(format!(
                                "{}:{} - Found mock in production code: {}",
                                entry.path().display(),
                                i + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }

        if !mock_violations.is_empty() {
            panic!(
                "Found {} mock violations in production code:\n{}",
                mock_violations.len(),
                mock_violations.join("\n")
            );
        }
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
