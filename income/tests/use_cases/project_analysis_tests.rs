mod project_analysis_tests {
    use test_context::TestContext;
    
    struct ProjectAnalysisContext {
        context_manager: ContextManager,
        mcp_client: McpClient,
        test_project: TestProject,
    }
    
    impl TestContext for ProjectAnalysisContext {
        fn setup() -> Self {
            let test_project = TestProject::new()
                .with_cargo_toml(TEST_CARGO_TOML)
                .with_rust_modules(TEST_MODULE_STRUCTURE)
                .with_dependencies();
                
            Self {
                context_manager: ContextManager::new(),
                mcp_client: McpClient::new(),
                test_project,
            }
        }
    }
    
    mod structure_analysis_tests {
        use super::*;
        
        #[tokio::test]
        async fn detects_project_structure_correctly() {
            let ctx = ProjectAnalysisContext::setup();
            
            let analysis = ctx.context_manager
                .analyze_project_structure(&ctx.test_project)
                .await?;
                
            assert_project_structure(&analysis, ExpectedStructure {
                modules: vec!["core", "utils", "api"],
                entry_points: vec!["main.rs", "lib.rs"],
                test_directories: vec!["tests/"],
            });
        }
    }
    
    mod language_detection_tests {
        use super::*;
        
        #[tokio::test]
        async fn identifies_languages_correctly() {
            let ctx = ProjectAnalysisContext::setup();
            
            let languages = ctx.context_manager
                .detect_languages(&ctx.test_project)
                .await?;
                
            assert_eq!(languages, vec![
                ("Rust", 0.80),
                ("Markdown", 0.15),
                ("TOML", 0.05),
            ]);
        }
    }
    
    mod dependency_analysis_tests {
        use super::*;
        
        #[tokio::test]
        async fn analyzes_dependencies_correctly() {
            let ctx = ProjectAnalysisContext::setup();
            
            let deps = ctx.context_manager
                .analyze_dependencies(&ctx.test_project)
                .await?;
                
            assert_dependency_graph(&deps, ExpectedDependencies {
                direct: vec!["tokio", "serde"],
                dev: vec!["mockall", "criterion"],
                build: vec!["build-deps"],
            });
        }
    }
} 