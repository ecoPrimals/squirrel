# Squirrel AI Tools

This crate provides integrations with various AI services like OpenAI, Anthropic, and Google Gemini for the Squirrel system. It handles authentication, rate limiting, prompt management, and response processing.

## Features

- Unified interface for multiple AI providers
- Support for chat-based API endpoints
- Tool calling / function calling support
- Streaming responses
- Rate limiting and error handling
- Secure credentials management

## Usage

Basic usage example:

```rust
use squirrel_ai_tools::{prelude::*, clients};

#[tokio::main]
async fn main() -> Result<()> {
    // Create an OpenAI client with your API key
    let client = clients::openai("your-api-key");
    
    // Create a chat request
    let request = ChatRequest::new()
        .add_system("You are a helpful assistant.")
        .add_user("What is the capital of France?")
        .with_model("gpt-3.5-turbo");
    
    // Get a response
    let response = client.chat(request).await?;
    
    // Print the response
    println!("Response: {}", response.message.content.unwrap_or_default());
    
    Ok(())
}
```

### Using tools/functions

```rust
use squirrel_ai_tools::{prelude::*, clients};

#[tokio::main]
async fn main() -> Result<()> {
    let client = clients::openai("your-api-key");
    
    // Define a tool
    let get_weather = FunctionDefinition {
        name: "get_weather".to_string(),
        description: "Get the current weather in a location".to_string(),
        parameters: ParameterSchema::object()
            .with_property(
                "location", 
                PropertySchema::string("The city and state, e.g. San Francisco, CA"), 
                true
            ),
    };
    
    let tools = vec![Tool::function(get_weather)];
    
    // Create a chat request with tools
    let request = ChatRequest::new()
        .add_user("What's the weather like in Paris, France?")
        .with_model("gpt-4-turbo-preview")
        .with_tools(tools);
    
    let response = client.chat(request).await?;
    
    // Handle tool calls
    if let Some(tool_calls) = response.message.tool_calls {
        for tool_call in tool_calls {
            println!("Tool call: {}", tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);
            
            // Parse the arguments and execute the function
            // ...
            
            // Send the function result back to the AI
            // ...
        }
    }
    
    Ok(())
}
```

### Streaming responses

```rust
use futures::StreamExt;
use squirrel_ai_tools::{prelude::*, clients};

#[tokio::main]
async fn main() -> Result<()> {
    let client = clients::openai("your-api-key");
    
    // Create a chat request with streaming enabled
    let mut params = ModelParameters::new();
    params.stream = Some(true);
    
    let request = ChatRequest::new()
        .add_user("Write a short story about a squirrel.")
        .with_parameters(params);
    
    // Get a streaming response
    let mut stream = client.chat_stream(request).await?;
    
    // Process the stream
    while let Some(chunk_result) = stream.inner.next().await {
        let chunk = chunk_result?;
        
        if let Some(content) = &chunk.content {
            print!("{}", content);
        }
        
        if chunk.is_final {
            println!("\n[Generation complete]");
        }
    }
    
    Ok(())
}
```

## Supported Providers

- OpenAI
- Anthropic (Claude) - Coming soon
- Google Gemini - Coming soon

## License

This crate is part of the Squirrel project. 