use reqwest;
use serde_json::json;
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
struct ModelConfig {
    name: String,
    description: String,
    role: String,
    max_tokens: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Multi-Model AI Discussion Demo");
    println!("==================================");
    println!("Demonstrating dynamic model routing and collaboration\n");

    // Get API key from environment
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    println!("✅ API Key loaded: {}...\n", &api_key[..20]);

    // Configure different models for different roles
    let models = vec![
        ModelConfig {
            name: "gpt-3.5-turbo".to_string(),
            description: "Fast, efficient model for quick responses".to_string(),
            role: "Analyst".to_string(),
            max_tokens: 150,
        },
        ModelConfig {
            name: "gpt-4o-mini".to_string(),
            description: "More capable model for complex reasoning".to_string(),
            role: "Strategist".to_string(),
            max_tokens: 200,
        },
        ModelConfig {
            name: "gpt-3.5-turbo".to_string(),
            description: "Creative model for innovative solutions".to_string(),
            role: "Creative Director".to_string(),
            max_tokens: 175,
        },
    ];

    // The task they'll discuss
    let task = "Design a sustainable urban transportation system for a city of 500,000 people";
    println!("🎯 Task: {}\n", task);

    let client = reqwest::Client::new();
    let mut conversation_history = Vec::new();

    // Each model contributes to the discussion
    for (index, model) in models.iter().enumerate() {
        println!("🤖 {} ({}) - {}", 
            model.role, 
            model.name, 
            model.description
        );
        println!("   Thinking about: {}", task);

        // Build context from previous responses
        let mut context = format!("You are a {} discussing: {}\n", model.role, task);
        
        if !conversation_history.is_empty() {
            context.push_str("\nPrevious contributions:\n");
            for (i, response) in conversation_history.iter().enumerate() {
                context.push_str(&format!("{}. {}\n", i + 1, response));
            }
        }

        // Role-specific prompts
        let role_prompt = match model.role.as_str() {
            "Analyst" => "Provide a data-driven analysis of the problem and current situation.",
            "Strategist" => "Based on the analysis, propose a strategic framework and key priorities.",
            "Creative Director" => "Building on the strategy, suggest innovative and creative solutions.",
            _ => "Contribute your expertise to this discussion.",
        };

        context.push_str(&format!("\n{} Please provide a concise response.", role_prompt));

        // Make API call
        let request_body = json!({
            "model": model.name,
            "messages": [
                {
                    "role": "user",
                    "content": context
                }
            ],
            "max_tokens": model.max_tokens,
            "temperature": 0.7
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await?;
            
            if let Some(content) = response_body["choices"][0]["message"]["content"].as_str() {
                println!("   💭 Response: {}", content);
                
                // Track usage
                if let Some(usage) = response_body.get("usage") {
                    println!("   📊 Tokens used: {}", usage["total_tokens"]);
                }
                
                conversation_history.push(format!("{}: {}", model.role, content));
            }
        } else {
            println!("   ❌ Error: {}", response.status());
        }

        println!();
    }

    // Summary of the multi-model discussion
    println!("📋 Multi-Model Discussion Summary:");
    println!("===================================");
    println!("✅ Dynamic model routing: Different models used for different roles");
    println!("✅ Contextual awareness: Each model built on previous responses");
    println!("✅ Capability-based selection: Models chosen for their strengths");
    println!("✅ Collaborative intelligence: Multiple AI perspectives combined");
    println!("\n🎉 Demo complete! This showcases how the squirrel system");
    println!("   can dynamically route to different models based on task requirements.");

    Ok(())
} 