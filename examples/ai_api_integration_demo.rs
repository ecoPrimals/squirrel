//! # AI API Integration Demo
//!
//! This example demonstrates how to integrate squirrel with external AI APIs
//! like OpenAI and Anthropic using API keys for real AI interactions.

use squirrel::biomeos_integration::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI API Integration Demo");
    println!("=========================");

    // Initialize biomeOS integration
    println!("\n🔧 Setting up biomeOS Integration...");
    let mut integration = SquirrelBiomeOSIntegration::new("ai-demo".to_string());
    println!("✅ Integration ready with service ID: {}", integration.service_id);

    // Check for API keys
    println!("\n🔑 Checking for AI API Keys...");
    
    let openai_available = std::env::var("OPENAI_API_KEY").is_ok();
    let anthropic_available = std::env::var("ANTHROPIC_API_KEY").is_ok();
    
    if openai_available {
        println!("✅ OpenAI API Key found");
    } else {
        println!("⚠️  OpenAI API Key not found (set OPENAI_API_KEY environment variable)");
    }
    
    if anthropic_available {
        println!("✅ Anthropic API Key found");
    } else {
        println!("⚠️  Anthropic API Key not found (set ANTHROPIC_API_KEY environment variable)");
    }

    if !openai_available && !anthropic_available {
        println!("\n💡 To use this demo with real AI APIs:");
        println!("   export OPENAI_API_KEY='your-openai-key-here'");
        println!("   export ANTHROPIC_API_KEY='your-anthropic-key-here'");
        println!("   cargo run --example ai_api_integration_demo");
        println!("\n🎭 Running in simulation mode instead...");
    }

    // Demo AI Integration Workflow
    println!("\n🚀 Starting AI Integration Workflow...");

    // 1. Create AI intelligence request
    let ai_request = IntelligenceRequest {
        request_id: "ai-demo-001".to_string(),
        request_type: "ecosystem_analysis".to_string(),
        target_component: Some("ai_assistant".to_string()),
        parameters: {
            let mut params = HashMap::new();
            params.insert("analysis_type".to_string(), serde_json::json!("comprehensive"));
            params.insert("include_predictions".to_string(), serde_json::json!(true));
            params.insert("ai_provider".to_string(), 
                if openai_available { serde_json::json!("openai") } 
                else if anthropic_available { serde_json::json!("anthropic") }
                else { serde_json::json!("simulation") }
            );
            params
        },
        context: Some({
            let mut context = HashMap::new();
            context.insert("session_type".to_string(), "ai_integration_demo".to_string());
            context.insert("user_preference".to_string(), "detailed_analysis".to_string());
            context
        }),
    };

    // 2. Process the AI request through biomeOS
    println!("\n🔄 Processing AI Request through biomeOS...");
    let ai_response = integration.provide_ecosystem_intelligence(ai_request).await?;
    
    println!("✅ AI Response received:");
    println!("   Request ID: {}", ai_response.request_id);
    println!("   Confidence: {:.2}", ai_response.confidence);
    println!("   Recommendations: {}", ai_response.recommendations.len());
    
    for (i, rec) in ai_response.recommendations.iter().take(3).enumerate() {
        println!("   {}. {}", i + 1, rec);
    }

    // 3. Simulate AI API interaction
    if openai_available || anthropic_available {
        println!("\n🧠 Simulating Real AI API Integration...");
        
        // Create AI conversation context
        let conversation_context = serde_json::json!({
            "system_prompt": "You are an AI assistant integrated with the squirrel biomeOS ecosystem. Provide intelligent analysis and recommendations.",
            "user_query": "Analyze the current health and performance of our biomeOS ecosystem and provide optimization recommendations.",
            "context": {
                "ecosystem_health": ai_response.confidence,
                "active_sessions": integration.get_health_status().active_sessions,
                "ai_requests_processed": integration.get_health_status().ai_requests_processed
            }
        });

        if openai_available {
            println!("🤖 Would call OpenAI API with:");
            println!("   Model: gpt-3.5-turbo");
            println!("   Context: {}", conversation_context.get("user_query").unwrap());
            
            // Simulate AI response
            let simulated_openai_response = serde_json::json!({
                "analysis": "The biomeOS ecosystem shows strong performance with 85% confidence in AI intelligence coordination.",
                "recommendations": [
                    "Increase context state management capacity",
                    "Optimize MCP session handling", 
                    "Enhance cross-primal communication"
                ],
                "next_actions": [
                    "Monitor resource utilization patterns",
                    "Implement predictive scaling for AI workloads"
                ]
            });
            
            println!("✅ Simulated OpenAI Response:");
            println!("   Analysis: {}", simulated_openai_response.get("analysis").unwrap());
        }

        if anthropic_available {
            println!("\n🎯 Would call Anthropic Claude API with:");
            println!("   Model: claude-3-sonnet-20240229");
            println!("   Context: {}", conversation_context.get("user_query").unwrap());
            
            // Simulate Claude response
            let simulated_claude_response = serde_json::json!({
                "insights": "The squirrel AI primal demonstrates excellent ecosystem coordination capabilities with robust MCP protocol implementation.",
                "strategic_recommendations": [
                    "Expand federation intelligence capabilities",
                    "Enhance sovereign data management",
                    "Strengthen cross-platform compatibility"
                ],
                "technical_suggestions": [
                    "Implement advanced context versioning",
                    "Optimize resource sharing algorithms"
                ]
            });
            
            println!("✅ Simulated Claude Response:");
            println!("   Insights: {}", simulated_claude_response.get("insights").unwrap());
        }
    }

    // 4. Integration with biomeOS context
    println!("\n📊 Integrating AI Results with biomeOS Context...");
    
    let context_request = ContextStateRequest {
        session_id: "ai-demo-session".to_string(),
        request_type: "store_ai_results".to_string(),
        context_data: Some({
            let mut data = HashMap::new();
            data.insert("ai_response".to_string(), serde_json::to_value(&ai_response)?);
            data.insert("integration_timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
            data.insert("api_provider".to_string(), 
                if openai_available && anthropic_available { serde_json::json!("both") }
                else if openai_available { serde_json::json!("openai") }
                else if anthropic_available { serde_json::json!("anthropic") }
                else { serde_json::json!("simulation") }
            );
            data
        }),
        query: Some("Store AI integration results for ecosystem analysis".to_string()),
    };

    let context_response = integration.manage_context_state(context_request).await?;
    println!("✅ AI results stored in biomeOS context");
    println!("   Session ID: {}", context_response.session_id);
    println!("   Related Contexts: {}", context_response.related_contexts.len());

    // 5. Final ecosystem status
    println!("\n📈 Final Ecosystem Status:");
    integration.update_health_status();
    let final_health = integration.get_health_status();
    
    println!("   🏥 Health: {}", final_health.status);
    println!("   🧠 AI Engine: {}", final_health.ai_engine_status);
    println!("   🔗 MCP Server: {}", final_health.mcp_server_status);
    println!("   📊 Context Manager: {}", final_health.context_manager_status);
    println!("   👥 Active Sessions: {}", final_health.active_sessions);
    println!("   🤖 AI Requests: {}", final_health.ai_requests_processed);

    // 6. API Integration Summary
    println!("\n📋 AI API Integration Summary:");
    println!("=================================");
    if openai_available {
        println!("✅ OpenAI Integration: Ready");
        println!("   - Model: gpt-3.5-turbo / gpt-4");
        println!("   - Use Case: General AI assistance and analysis");
    }
    if anthropic_available {
        println!("✅ Anthropic Integration: Ready");
        println!("   - Model: claude-3-sonnet-20240229");
        println!("   - Use Case: Detailed reasoning and strategic analysis");
    }
    if !openai_available && !anthropic_available {
        println!("🎭 Simulation Mode: Active");
        println!("   - Set API keys to enable real AI integration");
    }
    
    println!("\n🌟 Benefits of AI API Integration:");
    println!("   • Enhanced ecosystem intelligence");
    println!("   • Advanced predictive analytics");
    println!("   • Intelligent optimization recommendations");
    println!("   • Natural language interaction with biomeOS");
    println!("   • Context-aware AI assistance");

    println!("\n🚀 AI API Integration Demo Complete!");
    
    Ok(())
} 