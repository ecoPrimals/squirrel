//! Entry point for the Squirrel CLI application.

use std::sync::{Arc, Mutex};

use squirrel_commands::CommandRegistry;
use squirrel_cli::{
    commands::register_commands,
    mcp::MCPServer
};

/// Squirrel CLI application
fn main() {
    // Configure logging
    setup_logging().expect("Failed to setup logging");

    // Handle command-line arguments
    let matches = squirrel_cli::commands::create_cli().get_matches();

    // Create command registry
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    
    // Register commands with the registry
    register_commands(&mut registry.lock().unwrap());

    // Process commands
    if let Some(("mcp", sub_m)) = matches.subcommand() {
        // Start MCP server
        if let Some(sub_matches) = sub_m.subcommand_matches("server") {
            let host = sub_matches.get_one::<String>("host").unwrap_or(&"localhost".to_string()).clone();
            let port = *sub_matches.get_one::<u16>("port").unwrap_or(&7777);
            
            println!("Starting MCP server on {}:{}", host, port);
            
            let server = MCPServer::new(host.clone(), port, registry.lock().unwrap().clone());
            
            if let Err(err) = server.start() {
                eprintln!("Failed to start MCP server: {}", err);
                std::process::exit(1);
            }
        }
    } else if let Some((cmd, args)) = matches.subcommand() {
        // Extract args as strings
        let args_vec = args.get_many::<String>("")
            .map(|values| values.map(|v| v.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Execute the command
        let registry_guard = registry.lock().unwrap();
        match registry_guard.execute(cmd, &args_vec) {
            Ok(result) => {
                println!("{}", result);
                std::process::exit(0);
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    } else {
        // Show help
        println!("Use --help to see available commands");
        std::process::exit(0);
    }
}

/// Configure logging for the application
fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    
    Ok(())
} 