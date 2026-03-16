// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Build script for squirrel-mcp — rerun detection only.

fn main() {
    println!("cargo:rerun-if-changed=src/");
}
