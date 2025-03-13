mod error_resolution_tests {
    use test_context::TestContext;
    
    struct ErrorResolutionContext {
        error_handler: ErrorHandler,
        ui_client: UiClient,
        test_errors: Vec<TestError>,
    }
    
    impl TestContext for ErrorResolutionContext {
        fn setup() -> Self {
            let test_errors = vec![
                TestError::compiler("E0308", "mismatched types"),
                TestError::runtime("panic", "index out of bounds"),
                TestError::logical("infinite loop detected"),
            ];
            
            Self {
                error_handler: ErrorHandler::new(),
                ui_client: UiClient::new(),
                test_errors,
            }
        }
    }
    
    mod error_explanation_tests {
        use super::*;
        
        #[tokio::test]
        async fn explains_compiler_errors_clearly() {
            let ctx = ErrorResolutionContext::setup();
            
            let explanation = ctx.error_handler
                .explain_error(&ctx.test_errors[0])
                .await?;
                
            assert_explanation_clarity(&explanation, ClarityMetrics {
                understandability: 0.9,
                context_relevance: 0.85,
                action_items: 0.9,
            });
        }
    }
    
    mod fix_suggestion_tests {
        use super::*;
        
        #[tokio::test]
        async fn suggests_valid_fixes() {
            let ctx = ErrorResolutionContext::setup();
            
            let fixes = ctx.error_handler
                .suggest_fixes(&ctx.test_errors[0])
                .await?;
                
            assert_fix_validity(&fixes, ValidityMetrics {
                applicability: 0.85,
                correctness: 0.9,
                safety: 0.95,
            });
        }
    }
    
    mod interactive_resolution_tests {
        use super::*;
        
        #[tokio::test]
        async fn guides_through_error_resolution() {
            let ctx = ErrorResolutionContext::setup();
            
            let resolution = ctx.error_handler
                .start_interactive_resolution(&ctx.test_errors[0])
                .await?;
                
            assert_resolution_effectiveness(&resolution, EffectivenessMetrics {
                step_clarity: 0.9,
                user_guidance: 0.85,
                resolution_success: 0.9,
            });
        }
    }
} 