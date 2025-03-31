# Refactoring Plan for `mcp` Crate

This document outlines the steps to refactor the monolithic structure of the `mcp` crate for improved modularity, maintainability, and organization.

## Current State Analysis

- The `src/lib.rs` file acts primarily as a re-export facade but also contains unusual basic arithmetic functions and tests.
- There's a potentially redundant `src/lib/adapter.rs` alongside the main `src/adapter/mod.rs`.
- The `src/ui.rs` file (357 lines) likely handles multiple responsibilities and should be broken down.
- Other directories (`plugins/`, `security/`, `tests/`, `bin/`) need review for structure and potential refactoring.

## Refactoring Steps

1.  **Consolidate Adapter Logic:**
    *   Investigate `src/lib/adapter.rs`.
        *   If it contains necessary logic, merge it into `src/adapter/mod.rs`.
        *   If it's redundant or unused, remove the file and the `src/lib/` directory.
    *   Ensure `src/adapter/mod.rs` contains all core adapter logic.
    *   If `src/adapter/mod.rs` becomes too large, break it down into submodules within `src/adapter/` (e.g., `src/adapter/credentials.rs`, `src/adapter/interface.rs`). Update `src/adapter/mod.rs` with `mod` declarations.

2.  **Clean up `src/lib.rs`:**
    *   Remove the basic arithmetic functions (`add`, `multiply`, `subtract`, `divide`) and their corresponding tests.
    *   If these functions are needed elsewhere, move them to a dedicated utility module (e.g., `src/utils.rs` or `src/math_utils.rs`).
    *   Review all re-exports for necessity and organization. Verify the contents of the `prelude` module.

3.  **Refactor `src/ui.rs`:**
    *   Analyze the responsibilities within `src/ui.rs`.
    *   Create a new directory `src/ui/`.
    *   Move `src/ui.rs` to `src/ui/mod.rs`.
    *   Break down the logic within `src/ui/mod.rs` into smaller, focused modules within the `src/ui/` directory (e.g., `src/ui/components.rs`, `src/ui/state.rs`, `src/ui/handlers.rs`).
    *   Update `mod` declarations in `src/ui/mod.rs` to include the new submodules.

4.  **Review Top-Level Modules:**
    *   Examine the contents and structure of `plugins/`, `security/`, `tests/`, and `bin/`.
    *   Apply similar refactoring principles if large files or illogical groupings are found.
    *   **TODO:** The `src/plugins/` directory is complex and contains many large files. It requires further refactoring into subdirectories (e.g., `registry`, `lifecycle`, `marketplace`, `loader`) as a follow-up task.
    *   The `src/security/` directory has been cleaned up (removed `.clean` and `.new` files).
    *   The `src/tests/` and `src/bin/` directories appear reasonably structured for now.

5.  **Update `src/main.rs` and `src/lib.rs`:**
    *   Adjust `mod` declarations and `use` statements in `src/main.rs` and `src/lib.rs` to reflect the new module structure, especially referencing `mod ui;` instead of `mod ui;` directly if `src/ui.rs` is moved.

6.  **Verification:**
    *   Run `cargo fmt` to ensure consistent formatting.
    *   Run `cargo clippy` to check for lints and potential issues.
    *   Run `cargo test` to confirm all tests pass.
    *   Run `cargo build` to ensure the crate compiles successfully.
    *   Address any errors or warnings that arise from these steps.

## Post-Refactoring Goals

- A clearer, more standard Rust project structure.
- Reduced file sizes and improved code locality.
- Easier navigation and maintenance of the codebase.
- Better separation of concerns between modules. 