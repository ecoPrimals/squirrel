use std::sync::Arc;

use log::debug;
use squirrel_integration::mcp_ai_tools::McpAiToolsAdapter;

use super::state::AiChatWidgetState;

/// Send a message to the AI assistant and process the response
pub async fn send_message(
    adapter: &Arc<McpAiToolsAdapter>,
    state: &mut AiChatWidgetState
) -> Result<String, String> {
    // Check if there's a message to send
    if state.input.trim().is_empty() {
        return Ok(String::new());
    }
    
    // Update state
    state.is_sending = true;
    
    // Get the selected model and save relevant info before adding messages
    let model_name = match state.get_selected_model() {
        Ok(model) => {
            debug!("Selected model: {}", model.to_api_name());
            model.to_api_name().to_string()
        },
        Err(err) => {
            // Reset state since there's no valid model
            state.is_sending = false;
            state.generating_response = false;
            return Err(err);
        }
    };
    
    // Prepare the message to send
    let message = state.input.clone();
    
    // Add the user message to the chat history
    state.add_user_message(message.clone());
    
    // Generate a response with the specified model
    let result = adapter.generate_response(
        "default",  // Use a default conversation ID
        Some(model_name), 
        None, 
        None
    ).await;
    
    match result {
        Ok(response) => {
            // Add a new assistant message
            state.add_assistant_message(response.clone());
            
            // Clear the input only after successful response
            state.input.clear();
            
            // Reset state
            state.generating_response = false;
            state.is_sending = false;
            
            Ok(response)
        }
        Err(err) => {
            let error_message = err.to_string();
            handle_error(state, &error_message);
            
            // Always return an Err for API errors
            Err(error_message)
        }
    }
}

/// Handle an error from the API
fn handle_error(state: &mut AiChatWidgetState, err_msg: &str) {
    // Check if it's an API key error
    if err_msg.contains("API key") || 
       err_msg.contains("authentication") || 
       err_msg.contains("auth") {
        let error_msg = format!("Error: OpenAI API key is missing or invalid. Please add a valid API key in settings.");
        state.add_system_message(error_msg);
    } else {
        // General error
        let error_msg = format!("Error: {}", err_msg);
        state.add_system_message(error_msg);
    }
    
    // Reset state
    state.generating_response = false;
    state.is_sending = false;
} 