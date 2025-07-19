use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Dynamic Model Switching Demo");
    println!("===============================");
    println!("Showing how easy it is to change models without code changes\n");

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_KEY not set, using demo mode");
        "demo-key-not-functional".to_string()
    });
    let client = reqwest::Client::new();

    // Configuration-driven model selection
    // In a real system, this would come from config files or environment variables
    let model_configs = [
        ("gpt-3.5-turbo", "Fast model for quick responses"),
        ("gpt-4o-mini", "Advanced model for complex reasoning"),
        (
            "gpt-3.5-turbo",
            "Same model, different temperature for creativity",
        ),
    ];

    let task = "Explain quantum computing in simple terms";
    println!("🎯 Task: {task}\n");

    for (i, (model_name, description)) in model_configs.iter().enumerate() {
        println!("🤖 Model {}: {} - {}", i + 1, model_name, description);

        // Different temperatures for different purposes
        let temperature = match i {
            0 => 0.3, // More factual
            1 => 0.5, // Balanced
            2 => 0.8, // More creative
            _ => 0.5,
        };

        let request_body = json!({
            "model": model_name,
            "messages": [
                {
                    "role": "user",
                    "content": format!("{} Use {} approach.", task,
                        match i {
                            0 => "a technical",
                            1 => "a balanced",
                            2 => "a creative and analogical",
                            _ => "a standard"
                        }
                    )
                }
            ],
            "max_tokens": 150,
            "temperature": temperature
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await?;

            if let Some(content) = response_body["choices"][0]["message"]["content"].as_str() {
                println!("   💭 Response: {content}");

                if let Some(usage) = response_body.get("usage") {
                    println!(
                        "   📊 Tokens: {} | Temperature: {}",
                        usage["total_tokens"], temperature
                    );
                }
            }
        } else {
            println!("   ❌ Error: {}", response.status());
        }

        println!();
    }

    println!("🎉 Dynamic Model Switching Features:");
    println!("====================================");
    println!("✅ Configuration-driven model selection");
    println!("✅ Runtime model switching (no code changes)");
    println!("✅ Per-request parameter customization");
    println!("✅ Same interface, different models");
    println!("✅ Easy to add new models or providers");
    println!("\n💡 In production, models could be switched via:");
    println!("   - Environment variables");
    println!("   - Configuration files");
    println!("   - API parameters");
    println!("   - User preferences");
    println!("   - Cost optimization");
    println!("   - Performance requirements");

    Ok(())
}
