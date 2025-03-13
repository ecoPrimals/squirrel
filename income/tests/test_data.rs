use std::collections::HashMap;

// Test data constants
pub const TEST_RUST_CODE: &str = r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#;

pub const TEST_README: &str = r#"
# Test Project

This is a test project for validation.

## Features
- Feature 1
- Feature 2

## Usage
```rust
fn main() {
    // Example code
}
```
"#;

pub const TEST_CARGO_TOML: &str = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
mockall = "0.11"
criterion = "0.5"
"#;

pub const TEST_MODULE_STRUCTURE: &[(&str, &str)] = &[
    ("src/main.rs", "fn main() {}"),
    ("src/lib.rs", "pub mod core;"),
    ("src/core/mod.rs", "pub mod utils;"),
    ("src/core/utils.rs", "pub fn helper() {}"),
];

// Test types and traits
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub completeness: f64,
    pub accuracy: f64,
    pub clarity: f64,
}

#[derive(Debug, Clone)]
pub struct RelevanceMetrics {
    pub context_awareness: f64,
    pub practicality: f64,
    pub best_practices: f64,
}

#[derive(Debug, Clone)]
pub struct ClarityMetrics {
    pub understandability: f64,
    pub context_relevance: f64,
    pub action_items: f64,
}

#[derive(Debug, Clone)]
pub struct ValidityMetrics {
    pub applicability: f64,
    pub correctness: f64,
    pub safety: f64,
}

#[derive(Debug, Clone)]
pub struct EffectivenessMetrics {
    pub step_clarity: f64,
    pub user_guidance: f64,
    pub resolution_success: f64,
}

#[derive(Debug, Clone)]
pub struct ExpectedStructure {
    pub modules: Vec<String>,
    pub entry_points: Vec<String>,
    pub test_directories: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExpectedDependencies {
    pub direct: Vec<String>,
    pub dev: Vec<String>,
    pub build: Vec<String>,
}

// Test assertion helpers
pub fn assert_explanation_quality(explanation: &str, metrics: &QualityMetrics) -> bool {
    // Check minimum length
    if explanation.len() < 50 {
        return false;
    }
    
    // Check for key components
    let has_context = explanation.contains("context");
    let has_analysis = explanation.contains("analysis");
    let has_recommendation = explanation.contains("recommendation");
    
    // Check metrics
    let meets_metrics = metrics.coverage >= 0.8 && metrics.accuracy >= 0.8;
    
    has_context && has_analysis && has_recommendation && meets_metrics
}

pub fn assert_suggestions_relevance(suggestions: &[String], metrics: &RelevanceMetrics) -> bool {
    // Check minimum number of suggestions
    if suggestions.is_empty() {
        return false;
    }
    
    // Check each suggestion
    let valid_suggestions = suggestions.iter().all(|s| {
        s.len() >= 20 && // Minimum length
        !s.contains("TODO") && // No TODOs
        s.contains("should") // Action-oriented
    });
    
    // Check metrics
    let meets_metrics = metrics.relevance_score >= 0.7 && metrics.practicality >= 0.7;
    
    valid_suggestions && meets_metrics
}

pub fn assert_explanation_clarity(explanation: &str, metrics: &ClarityMetrics) -> bool {
    // Check readability
    let sentences: Vec<&str> = explanation.split('.').collect();
    let avg_sentence_length: f64 = sentences.iter()
        .map(|s| s.split_whitespace().count() as f64)
        .sum::<f64>() / sentences.len() as f64;
    
    // Check for technical terms
    let technical_terms = ["implementation", "function", "method", "struct", "trait"];
    let has_technical_terms = technical_terms.iter().any(|term| explanation.contains(term));
    
    // Check metrics
    let meets_metrics = metrics.readability >= 0.7 && metrics.technical_accuracy >= 0.8;
    
    avg_sentence_length <= 20.0 && has_technical_terms && meets_metrics
}

pub fn assert_fix_validity(fixes: &[String], metrics: &ValidityMetrics) -> bool {
    // Check minimum number of fixes
    if fixes.is_empty() {
        return false;
    }
    
    // Check each fix
    let valid_fixes = fixes.iter().all(|f| {
        f.len() >= 10 && // Minimum length
        !f.contains("TODO") && // No TODOs
        f.contains("fix") // Action-oriented
    });
    
    // Check metrics
    let meets_metrics = metrics.correctness >= 0.8 && metrics.completeness >= 0.8;
    
    valid_fixes && meets_metrics
}

pub fn assert_resolution_effectiveness(resolution: &str, metrics: &EffectivenessMetrics) -> bool {
    // Check resolution completeness
    let has_root_cause = resolution.contains("root cause");
    let has_solution = resolution.contains("solution");
    let has_prevention = resolution.contains("prevent");
    
    // Check metrics
    let meets_metrics = metrics.effectiveness >= 0.8 && metrics.durability >= 0.7;
    
    has_root_cause && has_solution && has_prevention && meets_metrics
}

pub fn assert_project_structure(analysis: &HashMap<String, String>, expected: &ExpectedStructure) -> bool {
    // Check required directories
    let has_src = analysis.contains_key("src");
    let has_tests = analysis.contains_key("tests");
    let has_docs = analysis.contains_key("docs");
    
    // Check required files
    let has_cargo_toml = analysis.contains_key("Cargo.toml");
    let has_readme = analysis.contains_key("README.md");
    
    // Check structure matches expected
    let matches_expected = analysis.keys().all(|k| expected.required_files.contains(k));
    
    has_src && has_tests && has_docs && has_cargo_toml && has_readme && matches_expected
}

pub fn assert_help_completeness(help_text: &str) -> bool {
    // Check required sections
    let has_usage = help_text.contains("USAGE:");
    let has_options = help_text.contains("OPTIONS:");
    let has_commands = help_text.contains("COMMANDS:");
    let has_examples = help_text.contains("EXAMPLES:");
    
    // Check minimum content
    let has_minimum_content = help_text.len() >= 200;
    
    has_usage && has_options && has_commands && has_examples && has_minimum_content
} 