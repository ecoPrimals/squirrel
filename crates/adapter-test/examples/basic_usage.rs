use adapter_test::{
    TestCommand, 
    CommandRegistryAdapter, 
    McpCommandAdapter, 
    CommandsPluginAdapter,
    Auth,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Registry Adapter Example ===");
    
    // Create registry adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Create test commands
    let hello_cmd = TestCommand::new(
        "hello", 
        "Says hello to the user", 
        "Hello, world!"
    );
    
    let echo_cmd = TestCommand::new(
        "echo", 
        "Echoes back the arguments", 
        "Echo"
    );
    
    // Register commands
    adapter.register_command(Box::new(hello_cmd))?;
    adapter.register_command(Box::new(echo_cmd))?;
    
    // List commands
    let commands = adapter.list_commands().await?;
    println!("Available commands: {:?}", commands);
    
    // Execute commands
    let hello_result = adapter.execute("hello", &[]).await?;
    println!("Hello command result: {}", hello_result);
    
    let echo_result = adapter.execute(
        "echo", 
        &["Hello".to_string(), "there!".to_string()]
    ).await?;
    println!("Echo command result: {}", echo_result);
    
    // Get help
    let help = adapter.get_help("hello").await?;
    println!("Help for hello: {}", help);
    
    println!("\n=== MCP Adapter Example ===");
    
    // Create MCP adapter
    let mcp_adapter = McpCommandAdapter::new();
    
    // Register regular and admin commands
    let regular_cmd = TestCommand::new(
        "regular", 
        "A regular command", 
        "Regular command result"
    );
    
    let admin_cmd = TestCommand::new(
        "admin-cmd", 
        "An admin-only command", 
        "Admin command result"
    );
    
    mcp_adapter.register_command(Box::new(regular_cmd))?;
    mcp_adapter.register_command(Box::new(admin_cmd))?;
    
    // List commands for admin
    let admin_commands = mcp_adapter.get_available_commands(
        Auth::User("admin".to_string(), "password".to_string())
    ).await?;
    println!("Commands available to admin: {:?}", admin_commands);
    
    // List commands for unauthenticated user
    let public_commands = mcp_adapter.get_available_commands(Auth::None).await?;
    println!("Commands available to public: {:?}", public_commands);
    
    // Execute regular command with authentication
    let result = mcp_adapter.execute_with_auth(
        "regular", 
        &[],
        Auth::User("admin".to_string(), "password".to_string())
    ).await?;
    println!("Regular command with auth: {}", result);
    
    // Execute regular command without authentication
    let result = mcp_adapter.execute_with_auth(
        "regular", 
        &[],
        Auth::None
    ).await?;
    println!("Regular command without auth: {}", result);
    
    // Try to execute admin command without authentication
    let result = mcp_adapter.execute_with_auth(
        "admin-cmd", 
        &[],
        Auth::None
    ).await;
    println!("Admin command without auth: {:?}", result);
    
    println!("\n=== Plugin Adapter Example ===");
    
    // Create plugin adapter
    let plugin_adapter = CommandsPluginAdapter::new();
    
    // Register plugin command
    let plugin_cmd = TestCommand::new(
        "plugin-cmd", 
        "A plugin command", 
        "Plugin command result"
    );
    
    plugin_adapter.register_command(Box::new(plugin_cmd))?;
    
    // Get available commands
    let commands = plugin_adapter.get_commands().await?;
    println!("Plugin commands: {:?}", commands);
    
    // Execute plugin command
    let result = plugin_adapter.execute_command(
        "plugin-cmd", 
        &["plugin".to_string(), "arg".to_string()]
    ).await?;
    println!("Plugin command result: {}", result);
    
    Ok(())
} 