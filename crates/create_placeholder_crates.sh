#!/bin/bash
# Create placeholder crates for the workspace

set -e

CRATES=(
  "core/core"
  "core/context"
  "core/plugins"
  "core/mcp"
  "integration/api-clients"
  "integration/context-adapter"
  "integration/mcp-pyo3-bindings"
  "integration/web"
  "services/app"
  "services/commands"
  "services/dashboard-core"
  "services/monitoring"
  "tools/ai-tools"
  "tools/cli"
  "tools/rule-system"
  "ui/ui-terminal"
  "ui/ui-tauri-react"
)

for crate in "${CRATES[@]}"; do
  mkdir -p "$crate/src"
  if [ ! -f "$crate/Cargo.toml" ]; then
    crate_name=$(basename "$crate")
    echo "[package]
name = \"squirrel-$crate_name\"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = \"Squirrel $crate_name component\"

[lints]
workspace = true

[dependencies]
" > "$crate/Cargo.toml"
  fi
  
  if [ ! -f "$crate/src/lib.rs" ]; then
    echo "// Placeholder for $crate_name" > "$crate/src/lib.rs"
  fi
  
  echo "Created placeholder for $crate"
done

echo "All placeholder crates created!" 