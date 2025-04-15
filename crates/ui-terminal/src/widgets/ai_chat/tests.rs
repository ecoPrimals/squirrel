#[cfg(test)]
mod tests {
    use crate::widgets::ai_chat::{
        models::AiModel,
        messages::ChatMessage,
    };

    #[test]
    fn test_ai_model_to_api_name() {
        assert_eq!(AiModel::Gpt35Turbo.to_api_name(), "gpt-3.5-turbo");
        assert_eq!(AiModel::Gpt4.to_api_name(), "gpt-4");
        assert_eq!(AiModel::Gpt4Turbo.to_api_name(), "gpt-4-turbo-preview");
        assert_eq!(AiModel::Gpt4Vision.to_api_name(), "gpt-4-vision-preview");
    }

    #[test]
    fn test_ai_model_display_name() {
        assert_eq!(AiModel::Gpt35Turbo.display_name(), "GPT-3.5 Turbo");
        assert_eq!(AiModel::Gpt4.display_name(), "GPT-4");
        assert_eq!(AiModel::Gpt4Turbo.display_name(), "GPT-4 Turbo");
        assert_eq!(AiModel::Gpt4Vision.display_name(), "GPT-4 Vision");
    }

    #[test]
    fn test_chat_message_roles() {
        let user_msg = ChatMessage::user("User message".to_string());
        let assistant_msg = ChatMessage::assistant("Assistant message".to_string());
        let system_msg = ChatMessage::system("System message".to_string());

        assert!(user_msg.is_user());
        assert!(!user_msg.is_assistant());
        assert!(!user_msg.is_system());

        assert!(!assistant_msg.is_user());
        assert!(assistant_msg.is_assistant());
        assert!(!assistant_msg.is_system());

        assert!(!system_msg.is_user());
        assert!(!system_msg.is_assistant());
        assert!(system_msg.is_system());
    }
} 