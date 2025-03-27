// Test modules
mod adapter_tests;
mod cli_end_to_end_tests;
mod concurrency_tests;
mod isolated_adapter_tests;
mod resource_limit_tests;
mod standalone_adapter_tests;

// Re-export test command module
pub use squirrel_cli::commands::test_command; 