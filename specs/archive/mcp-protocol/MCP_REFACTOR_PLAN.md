# MCP Crate Refactoring Plan

## Goals

*   Improve modularity and reduce coupling between components.
*   Simplify the codebase structure for better maintainability and understanding.
*   Reduce complexity by breaking down large modules and files.
*   Address identified technical debt (large `types.rs`, inconsistent structure, `allow` directives).
*   Establish clearer boundaries between core concerns (protocol, transport, security, state, etc.).

## Approach

This will be an aggressive refactoring process. Expect significant breakage during intermediate steps. We will focus on structural improvements first, then address resulting compilation errors.

## Proposed Steps

1.  **Investigate `src/mod.rs`:**
    *   Determine the purpose of the `src/mod.rs` file.
    *   Integrate its contents into `src/lib.rs` or delete it if redundant.

2.  **Refactor Centralized `types.rs`:**
    *   Identify groups of types related to specific modules (e.g., `transport`, `security`, `protocol`, `config`, `error`).
    *   Create `types.rs` files within relevant module directories (e.g., `src/transport/types.rs`, `src/security/types.rs`).
    *   Move type definitions from `src/types.rs` to these new locations.
    *   Update `use` statements across the crate accordingly.
    *   Aim to significantly reduce the size and scope of `src/types.rs`, potentially removing it entirely if all types are relocated.

3.  **Decompose Large Files/Modules:**
    *   **`src/client.rs` & `src/server.rs`:**
        *   Convert these top-level files into modules (`src/client/mod.rs`, `src/server/mod.rs`).
        *   Identify distinct responsibilities within each (e.g., connection handling, request processing, state management) and extract them into sub-modules/files within `src/client/` and `src/server/`.
    *   **`src/context_manager.rs`:** Analyze its responsibilities and break it down into smaller, more focused components if applicable.
    *   **Other Large Files:** Review other files identified as potentially large (e.g., `security/mod.rs`) and decompose if necessary.

4.  **Improve Module Hierarchy & Structure:**
    *   Review the flat module structure declared in `src/lib.rs`.
    *   Nest related modules where appropriate (e.g., `message`, `frame`, `message_router` under `protocol`; `session`, `port` under `transport`; `persistence`, `registry` under a potential `storage` module).
    *   Ensure module naming conventions are consistent.

5.  **Address Technical Debt (`allow` directives):**
    *   As modules are refactored, remove corresponding `allow` directives (Clippy, docs, dead code).
    *   Add necessary documentation (`missing_docs`, `missing_errors_doc`, `missing_panics_doc`).
    *   Fix Clippy warnings that become relevant.

6.  **Iterative Build & Fix:**
    *   After major structural changes (like moving types or restructuring modules), run `cargo build` frequently.
    *   Address the resulting compilation errors systematically.

## Expected Outcome

A more modular, understandable, and maintainable `mcp` crate with reduced technical debt, making future development and debugging easier. 