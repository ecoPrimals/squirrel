mod code_understanding_tests {
    use test_context::TestContext;
    
    struct CodeUnderstandingContext {
        command_system: CommandSystem,
        ui_client: UiClient,
        test_codebase: TempCodebase,
    }
    
    impl TestContext for CodeUnderstandingContext {
        fn setup() -> Self {
            // Initialize test codebase with known content
            let test_codebase = TempCodebase::new()
                .with_file("src/main.rs", TEST_RUST_CODE)
                .with_file("README.md", TEST_README);
                
            Self {
                command_system: CommandSystem::new(),
                ui_client: UiClient::new(),
                test_codebase,
            }
        }
        
        fn teardown(self) {
            self.test_codebase.cleanup();
        }
    }
    
    mod explanation_tests {
        use super::*;
        
        #[tokio::test]
        async fn explains_rust_function_correctly() {
            let ctx = CodeUnderstandingContext::setup();
            
            let explanation = ctx.command_system
                .execute_command("explain", ["src/main.rs:10:20"])
                .await?;
                
            assert_explanation_quality(&explanation, QualityMetrics {
                completeness: 0.8,
                accuracy: 0.9,
                clarity: 0.85,
            });
        }
        
        #[tokio::test]
        async fn handles_context_dependent_code() {
            let ctx = CodeUnderstandingContext::setup();
            
            let explanation = ctx.command_system
                .execute_command("explain", ["src/main.rs:30:40"])
                .await?;
                
            assert!(explanation.contains("imports"));
            assert!(explanation.contains("dependencies"));
        }
    }
    
    mod suggestion_tests {
        use super::*;
        
        #[tokio::test]
        async fn provides_relevant_code_suggestions() {
            let ctx = CodeUnderstandingContext::setup();
            
            let suggestions = ctx.command_system
                .execute_command("suggest", ["src/main.rs:15"])
                .await?;
                
            assert_suggestions_relevance(&suggestions, RelevanceMetrics {
                context_awareness: 0.8,
                practicality: 0.85,
                best_practices: 0.9,
            });
        }
    }
    
    mod help_system_tests {
        use super::*;
        
        #[tokio::test]
        async fn provides_accurate_command_help() {
            let ctx = CodeUnderstandingContext::setup();
            
            let help_text = ctx.command_system
                .execute_command("help", ["explain"])
                .await?;
                
            assert_help_completeness(&help_text);
        }
    }
} 