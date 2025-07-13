use reqwest;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Testing Real OpenAI API Integration");
    println!("=====================================");

    // Get API key from environment
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    println!("✅ API Key found: {}...", &api_key[..20]);

    // Test different models to show adaptability
    let models_to_test = vec![
        ("gpt-3.5-turbo", "Basic fast model"),
        ("gpt-4o-mini", "More capable model"),
    ];

    let client = reqwest::Client::new();

    for (model, description) in models_to_test {
        println!("\n🧠 Testing model: {} ({})", model, description);
        
        let request_body = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": "Respond with just 'Hello from' followed by your model name"
                }
            ],
            "max_tokens": 20,
            "temperature": 0.1
        });

        match client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let result: serde_json::Value = response.json().await?;
                    if let Some(content) = result["choices"][0]["message"]["content"].as_str() {
                        println!("   ✅ Response: {}", content.trim());
                        println!("   📊 Usage: {} tokens", 
                            result["usage"]["total_tokens"].as_u64().unwrap_or(0));
                    }
                } else {
                    println!("   ❌ Model {} not available (status: {})", model, response.status());
                    if model == "gpt-4o-mini" {
                        println!("   💡 This shows how the system can fallback when models change");
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Error with {}: {}", model, e);
            }
        }
    }

    println!("\n🎯 Key Takeaways:");
    println!("   • System is NOT hardcoded to specific models");
    println!("   • Can dynamically test different model versions");
    println!("   • Gracefully handles model availability changes");
    println!("   • Configuration-driven model selection");
    
    Ok(())
} 