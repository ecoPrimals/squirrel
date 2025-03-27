#!/bin/bash
# Run all adapter pattern examples

echo "===== Adapter Pattern Examples Demo ====="
echo

echo "1. Running Basic Demo (demo.rs)"
echo "--------------------------------------"
cargo run --bin demo
echo
echo "--------------------------------------"
echo

echo "2. Running Custom Command Example (custom_command.rs)"
echo "--------------------------------------"
cargo run --bin custom_command
echo
echo "--------------------------------------"
echo

echo "3. Running CLI Application Demo (cli_app.rs)"
echo "--------------------------------------"
echo "3.1 Help command:"
cargo run --bin cli_app help
echo
echo "3.2 Echo command:"
cargo run --bin cli_app echo "Hello from the adapter pattern!"
echo
echo "3.3 Greeting command:"
cargo run --bin cli_app greet formal "Rust Developer"
echo
echo "3.4 Secure command (should fail without authentication):"
cargo run --bin cli_app secure
echo
echo "3.5 Secure command with authentication:"
cargo run --bin cli_app login user password secure
echo
echo "--------------------------------------"
echo

echo "4. Auth Example (auth_example.rs) [Interactive - run separately]"
echo "--------------------------------------"
echo "To run the interactive auth example:"
echo "$ cargo run --bin auth_example"
echo "--------------------------------------"
echo

echo "===== End of Examples =====" 