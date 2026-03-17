// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! `OrExit<T>` — zero-panic binary entry point pattern.
//!
//! Absorbed from the ecosystem consensus (coralReef, groundSpring, neuralSpring,
//! sweetGrass, petalTongue, loamSpine). Wraps `Result<T, E>` with structured
//! exit codes and human-readable stderr messages instead of panicking.
//!
//! # Usage
//!
//! ```ignore
//! use universal_patterns::OrExit;
//!
//! fn main() {
//!     let config = load_config().or_exit("loading configuration");
//!     let socket = bind_socket(&config).or_exit("binding socket");
//! }
//! ```

use std::fmt;
use std::process;

/// Extension trait for `Result<T, E>` that exits the process on error
/// with a structured exit code and human-readable message.
pub trait OrExit<T> {
    /// Unwrap the result or exit with a structured error message.
    ///
    /// Prints `"fatal: {context}: {error}"` to stderr and exits with the
    /// appropriate exit code based on the error kind.
    fn or_exit(self, context: &str) -> T;

    /// Unwrap the result or exit with a specific exit code.
    fn or_exit_code(self, context: &str, code: i32) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                let msg = e.to_string();
                let code = exit_code_for_error(&msg);
                eprintln!("fatal: {context}: {e}");
                process::exit(code);
            }
        }
    }

    fn or_exit_code(self, context: &str, code: i32) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("fatal: {context}: {e}");
                process::exit(code);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Some(v) => v,
            None => {
                eprintln!("fatal: {context}: value was None");
                process::exit(exit_codes::ERROR);
            }
        }
    }

    fn or_exit_code(self, context: &str, code: i32) -> T {
        match self {
            Some(v) => v,
            None => {
                eprintln!("fatal: {context}: value was None");
                process::exit(code);
            }
        }
    }
}

/// Centralized UniBin exit codes — ecosystem standard.
///
/// Shared across all binary entry points (server, client, doctor, validate).
pub mod exit_codes {
    /// Clean exit
    pub const SUCCESS: i32 = 0;
    /// General error
    pub const ERROR: i32 = 1;
    /// Configuration error (missing env var, bad TOML, etc.)
    pub const CONFIG: i32 = 2;
    /// Network / socket error (bind, connect, DNS)
    pub const NETWORK: i32 = 3;
    /// Permission denied (file ACL, capability, auth)
    pub const PERMISSION: i32 = 4;
    /// Resource exhaustion (OOM, fd limit, disk full)
    pub const RESOURCE: i32 = 5;
    /// Interrupted by signal (SIGINT / SIGTERM)
    pub const INTERRUPTED: i32 = 130;
}

/// Heuristic: map error message content to an appropriate exit code.
fn exit_code_for_error(msg: &str) -> i32 {
    let lower = msg.to_lowercase();
    if lower.contains("config")
        || lower.contains("toml")
        || lower.contains("env")
        || lower.contains("parse")
    {
        exit_codes::CONFIG
    } else if lower.contains("socket")
        || lower.contains("connect")
        || lower.contains("bind")
        || lower.contains("network")
        || lower.contains("dns")
        || lower.contains("address")
    {
        exit_codes::NETWORK
    } else if lower.contains("permission") || lower.contains("denied") || lower.contains("auth") {
        exit_codes::PERMISSION
    } else if lower.contains("resource")
        || lower.contains("memory")
        || lower.contains("oom")
        || lower.contains("limit")
    {
        exit_codes::RESOURCE
    } else {
        exit_codes::ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_ok_passes_through() {
        let r: Result<i32, &str> = Ok(42);
        assert_eq!(r.or_exit("test"), 42);
    }

    #[test]
    fn option_some_passes_through() {
        let o: Option<i32> = Some(42);
        assert_eq!(o.or_exit("test"), 42);
    }

    #[test]
    fn exit_code_heuristic_config() {
        assert_eq!(
            exit_code_for_error("missing config file"),
            exit_codes::CONFIG
        );
        assert_eq!(exit_code_for_error("bad TOML syntax"), exit_codes::CONFIG);
        assert_eq!(exit_code_for_error("env var not set"), exit_codes::CONFIG);
    }

    #[test]
    fn exit_code_heuristic_network() {
        assert_eq!(
            exit_code_for_error("socket bind failed"),
            exit_codes::NETWORK
        );
        assert_eq!(exit_code_for_error("connect refused"), exit_codes::NETWORK);
    }

    #[test]
    fn exit_code_heuristic_permission() {
        assert_eq!(
            exit_code_for_error("permission denied"),
            exit_codes::PERMISSION
        );
    }

    #[test]
    fn exit_code_heuristic_resource() {
        assert_eq!(exit_code_for_error("out of memory"), exit_codes::RESOURCE);
    }

    #[test]
    fn exit_code_heuristic_generic() {
        assert_eq!(
            exit_code_for_error("something went wrong"),
            exit_codes::ERROR
        );
    }

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            exit_codes::SUCCESS,
            exit_codes::ERROR,
            exit_codes::CONFIG,
            exit_codes::NETWORK,
            exit_codes::PERMISSION,
            exit_codes::RESOURCE,
            exit_codes::INTERRUPTED,
        ];
        let unique: std::collections::HashSet<i32> = codes.iter().copied().collect();
        assert_eq!(unique.len(), codes.len(), "exit codes must be unique");
    }
}
