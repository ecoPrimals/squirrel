// Export the test modules
pub mod adapter_tests;
pub mod cli_end_to_end_tests;
pub mod concurrency_tests;
pub mod isolated_adapter_tests;
pub mod resource_limit_tests;
pub mod standalone_adapter_tests;

// Export the test command module conditionally
#[cfg(feature = "testing")]
pub use squirrel_cli::commands::test_command; 