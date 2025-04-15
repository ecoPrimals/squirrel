// crates/ui-terminal/tests/app_integration_core.rs
//! Main integration test module that includes other test modules

// Re-export the mocks for use in other test files
pub mod mocks;

// Include other test modules
#[path = "app_basic_test.rs"]
mod basic_tests;

#[path = "app_navigation_test.rs"]
mod navigation_tests;

#[path = "app_alerts_test.rs"]
mod alerts_tests;

#[path = "app_ai_chat_test.rs"]
mod ai_chat_tests;

#[path = "app_ai_chat_advanced_test.rs"]
mod ai_chat_advanced_tests;

#[path = "openai_integration_test.rs"]
mod openai_integration_tests; 