use adapter_pattern_examples::{
    Command, CommandAdapter, CommandError, CommandResult, RegistryAdapter,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

/// A simple calculator command that can add, subtract, multiply, and divide
#[derive(Debug)]
struct CalculatorCommand {
    name: String,
    description: String,
}

impl CalculatorCommand {
    fn new() -> Self {
        Self {
            name: "calc".to_string(),
            description: "Simple calculator (usage: calc [add|sub|mul|div] x y)".to_string(),
        }
    }

    fn calculate(&self, operation: &str, x: f64, y: f64) -> CommandResult<f64> {
        match operation {
            "add" => Ok(x + y),
            "sub" => Ok(x - y),
            "mul" => Ok(x * y),
            "div" => {
                if y == 0.0 {
                    Err(CommandError::ExecutionFailed(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(x / y)
                }
            }
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }
}

#[async_trait]
impl Command for CalculatorCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.len() != 3 {
            return Err(CommandError::ExecutionFailed(
                "Invalid arguments. Usage: calc [add|sub|mul|div] x y".to_string(),
            ));
        }

        let operation = &args[0];

        // Parse arguments
        let x = args[1]
            .parse::<f64>()
            .map_err(|e| CommandError::ExecutionFailed(format!("Invalid number: {}", e)))?;

        let y = args[2]
            .parse::<f64>()
            .map_err(|e| CommandError::ExecutionFailed(format!("Invalid number: {}", e)))?;

        // Calculate result
        let result = self.calculate(operation, x, y)?;

        // Format result
        Ok(format!("{} {} {} = {}", x, operation, y, result))
    }
}

/// A weather forecast command that returns the weather for a city
#[derive(Debug)]
struct WeatherCommand {
    name: String,
    description: String,
}

impl WeatherCommand {
    fn new() -> Self {
        Self {
            name: "weather".to_string(),
            description: "Get weather forecast for a city (usage: weather <city>)".to_string(),
        }
    }

    // This would use a real API in a production application
    fn get_weather(&self, city: &str) -> CommandResult<String> {
        // Simulate API response
        match city.to_lowercase().as_str() {
            "london" => Ok("London: 15°C, Cloudy".to_string()),
            "new york" => Ok("New York: 22°C, Sunny".to_string()),
            "tokyo" => Ok("Tokyo: 26°C, Rainy".to_string()),
            "sydney" => Ok("Sydney: 19°C, Partly cloudy".to_string()),
            _ => Ok(format!("{}: 20°C, Weather data not available", city)),
        }
    }
}

#[async_trait]
impl Command for WeatherCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Err(CommandError::ExecutionFailed(
                "City name required. Usage: weather <city>".to_string(),
            ));
        }

        let city = &args[0];
        self.get_weather(city)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Commands Demo ===\n");

    // Create registry adapter
    let adapter = RegistryAdapter::new();

    // Register custom commands
    adapter.register_command(Arc::new(CalculatorCommand::new()))?;
    adapter.register_command(Arc::new(WeatherCommand::new()))?;

    // List available commands
    let commands = adapter.list_commands().await?;
    println!("Available commands: {:?}", commands);

    // Get help for commands
    let calc_help = adapter.get_help("calc").await?;
    println!("\nHelp: {}", calc_help);

    let weather_help = adapter.get_help("weather").await?;
    println!("Help: {}", weather_help);

    // Execute calculator command
    println!("\n--- Calculator Command ---");

    // Add operation
    let add_result = adapter
        .execute_command(
            "calc",
            vec!["add".to_string(), "5".to_string(), "3".to_string()],
        )
        .await?;
    println!("Addition result: {}", add_result);

    // Multiply operation
    let mul_result = adapter
        .execute_command(
            "calc",
            vec!["mul".to_string(), "4".to_string(), "7".to_string()],
        )
        .await?;
    println!("Multiplication result: {}", mul_result);

    // Division operation
    let div_result = adapter
        .execute_command(
            "calc",
            vec!["div".to_string(), "10".to_string(), "2".to_string()],
        )
        .await?;
    println!("Division result: {}", div_result);

    // Division by zero (should fail)
    let div_zero = adapter
        .execute_command(
            "calc",
            vec!["div".to_string(), "10".to_string(), "0".to_string()],
        )
        .await;
    println!("Division by zero: {:?}", div_zero);

    // Execute weather command
    println!("\n--- Weather Command ---");

    let london_weather = adapter
        .execute_command("weather", vec!["London".to_string()])
        .await?;
    println!("{}", london_weather);

    let tokyo_weather = adapter
        .execute_command("weather", vec!["Tokyo".to_string()])
        .await?;
    println!("{}", tokyo_weather);

    Ok(())
}
