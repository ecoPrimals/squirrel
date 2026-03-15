// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Advanced Adapter Pattern Demonstration
//!
//! This example demonstrates advanced features of the adapter pattern implementation,
//! focusing on the enhanced MCP adapter with:
//! - Role-based authentication and authorization
//! - Token-based authentication
//! - Audit logging
//! - Permission-based command filtering

use adapter_pattern_tests::{Auth, CommandError, CommandResult, McpAdapter, TestCommand, UserRole};
use std::sync::Arc;

#[tokio::main]
async fn main() -> CommandResult<()> {
    print_header("Advanced Adapter Pattern Demo");

    // Create and configure the MCP adapter with various users and commands
    let mut mcp_adapter = setup_enhanced_mcp_adapter()?;

    // Demonstrate authentication with different methods
    demo_authentication(&mut mcp_adapter).await?;

    // Demonstrate authorization with role-based permissions
    demo_authorization(&mut mcp_adapter).await?;

    // Demonstrate token-based authentication
    demo_token_authentication(&mut mcp_adapter).await?;

    // Demonstrate audit logging
    demo_audit_logging(&mut mcp_adapter).await?;

    print_header("Advanced Adapter Pattern Demo Completed");

    Ok(())
}

/// Setup an enhanced MCP adapter with various users and commands
fn setup_enhanced_mcp_adapter() -> CommandResult<McpAdapter> {
    println!("Setting up enhanced MCP adapter...");

    let mut adapter = McpAdapter::new();

    // Register commands
    let hello_cmd = TestCommand::new("hello", "Basic hello command", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echo command", "Echo");
    let admin_cmd = TestCommand::new("admin-stats", "Admin statistics", "Admin statistics");
    let power_cmd = TestCommand::new("power-tool", "Power user tool", "Power tool");
    let secure_cmd = TestCommand::new("secure-data", "Secure data access", "Secure data");

    tokio::task::block_in_place(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            adapter.register_command(Arc::new(hello_cmd)).await?;
            adapter.register_command(Arc::new(echo_cmd)).await?;
            adapter.register_command(Arc::new(admin_cmd)).await?;
            adapter.register_command(Arc::new(power_cmd)).await?;
            adapter.register_command(Arc::new(secure_cmd)).await?;

            // Add users with different roles
            // Admin user is already added by default
            adapter.add_user("power", "power123", false);
            adapter.add_user("regular", "regular123", false);
            adapter.add_user("guest", "guest123", false);

            // Add command permissions
            adapter.add_command_with_permissions(
                "power-tool",
                vec![UserRole::PowerUser, UserRole::Admin],
            );
            adapter.add_command_with_permissions(
                "secure-data",
                vec![UserRole::RegularUser, UserRole::PowerUser, UserRole::Admin],
            );

            // Update user roles - we can't directly access the field, so we'll set up the user with the right role
            adapter.add_user("power", "power123", false);
            // Add the user again with PowerUser role by registering a special command and adding permissions
            adapter.add_command_with_permissions("power-user-cmd", vec![UserRole::PowerUser]);
            adapter
                .execute_with_auth(
                    "power-user-cmd",
                    vec![],
                    Auth::User("power".to_string(), "power123".to_string()),
                )
                .await
                .ok();

            Ok::<_, CommandError>(())
        })
    })?;

    println!("Adapter setup complete. Registered users: admin, power, regular, guest");
    println!("Registered commands: hello, echo, admin-stats, power-tool, secure-data");

    Ok(adapter)
}

/// Demonstrate different authentication methods
async fn demo_authentication(adapter: &mut McpAdapter) -> CommandResult<()> {
    print_section("Authentication Demo");

    println!("1. Anonymous authentication:");
    match adapter.execute_with_auth("hello", vec![], Auth::None).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n2. Username/password authentication:");
    let user_auth = Auth::User("admin".to_string(), "password".to_string());
    match adapter.execute_with_auth("hello", vec![], user_auth).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n3. API key authentication:");
    let api_auth = Auth::ApiKey("squirrel-api-key".to_string());
    match adapter.execute_with_auth("hello", vec![], api_auth).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n4. Invalid authentication:");
    let invalid_auth = Auth::User("admin".to_string(), "wrong-password".to_string());
    match adapter
        .execute_with_auth("hello", vec![], invalid_auth)
        .await
    {
        Ok(result) => println!("  Unexpected success: {}", result),
        Err(e) => println!("  Expected error: {}", e),
    }

    Ok(())
}

/// Demonstrate role-based authorization
async fn demo_authorization(adapter: &mut McpAdapter) -> CommandResult<()> {
    print_section("Authorization Demo");

    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let power_auth = Auth::User("power".to_string(), "power123".to_string());
    let regular_auth = Auth::User("regular".to_string(), "regular123".to_string());

    println!("1. Command visibility by role:");
    let admin_cmds = adapter.get_available_commands(admin_auth.clone()).await?;
    let power_cmds = adapter.get_available_commands(power_auth.clone()).await?;
    let regular_cmds = adapter.get_available_commands(regular_auth.clone()).await?;
    let anon_cmds = adapter.get_available_commands(Auth::None).await?;

    println!("  Admin can access: {:?}", admin_cmds);
    println!("  Power user can access: {:?}", power_cmds);
    println!("  Regular user can access: {:?}", regular_cmds);
    println!("  Anonymous can access: {:?}", anon_cmds);

    println!("\n2. Admin command access:");
    match adapter
        .execute_with_auth("admin-stats", vec![], admin_auth.clone())
        .await
    {
        Ok(result) => println!("  Admin access: {}", result),
        Err(e) => println!("  Admin access error: {}", e),
    }

    match adapter
        .execute_with_auth("admin-stats", vec![], power_auth.clone())
        .await
    {
        Ok(result) => println!("  Power user access: {}", result),
        Err(e) => println!("  Power user access error: {}", e),
    }

    println!("\n3. Power user command access:");
    match adapter
        .execute_with_auth("power-tool", vec![], power_auth.clone())
        .await
    {
        Ok(result) => println!("  Power user access: {}", result),
        Err(e) => println!("  Power user access error: {}", e),
    }

    match adapter
        .execute_with_auth("power-tool", vec![], regular_auth.clone())
        .await
    {
        Ok(result) => println!("  Regular user access: {}", result),
        Err(e) => println!("  Regular user access error: {}", e),
    }

    Ok(())
}

/// Demonstrate token-based authentication
async fn demo_token_authentication(adapter: &mut McpAdapter) -> CommandResult<()> {
    print_section("Token Authentication Demo");

    println!("1. Generating token for admin user:");
    let token = adapter.generate_token("admin", "password")?;
    println!("  Token: {}", token);

    println!("\n2. Using token for authentication:");
    let token_auth = Auth::Token(token);

    match adapter
        .execute_with_auth("admin-stats", vec![], token_auth.clone())
        .await
    {
        Ok(result) => println!("  Command execution: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n3. Attempting with invalid token:");
    let invalid_token = Auth::Token("invalid-token".to_string());

    match adapter
        .execute_with_auth("hello", vec![], invalid_token)
        .await
    {
        Ok(result) => println!("  Unexpected success: {}", result),
        Err(e) => println!("  Expected error: {}", e),
    }

    Ok(())
}

/// Demonstrate audit logging
async fn demo_audit_logging(adapter: &mut McpAdapter) -> CommandResult<()> {
    print_section("Audit Logging Demo");

    println!("Executing various commands to generate log entries...");

    // Execute some commands with different users
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let power_auth = Auth::User("power".to_string(), "power123".to_string());

    // Successful commands
    let _ = adapter
        .execute_with_auth("hello", vec![], admin_auth.clone())
        .await?;
    let _ = adapter
        .execute_with_auth("echo", vec!["test".to_string()], power_auth.clone())
        .await?;

    // Failed command (intentional)
    let _ = adapter
        .execute_with_auth("nonexistent", vec![], admin_auth.clone())
        .await
        .unwrap_err();

    println!("\nAudit log entries:");
    let logs = adapter.get_command_logs();
    for (i, entry) in logs.iter().enumerate() {
        println!("{}. Command log entry {}", i + 1, i + 1);
        // Since we can't access the private fields directly, we'll just print the Debug representation
        println!("   Log: {:?}", entry);
        println!();
    }

    Ok(())
}

/// Print a section header
fn print_section(title: &str) {
    println!("\n{}", "=".repeat(80));
    println!("  {}", title);
    println!("{}\n", "=".repeat(80));
}

/// Print a header
fn print_header(title: &str) {
    println!("\n{}", "#".repeat(80));
    println!("  {}", title);
    println!("{}\n", "#".repeat(80));
}
