use std::path::PathBuf;
use squirrel_rule_system::{
    parser::parse_rule_file,
    utils::{create_rule_template, rule_matches_context},
    directory::{create_rule_directory_manager_with_root_dir, RuleDirectoryConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable logging
    tracing_subscriber::fmt::init();
    
    println!("Rule System Parser Example");
    println!("=========================\n");
    
    // Get the examples directory path
    let examples_dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("examples/rules");
        path.to_string_lossy().to_string()
    }));
    
    println!("Loading rules from: {}\n", examples_dir.display());
    
    // Create a rule directory manager
    let _config = RuleDirectoryConfig {
        root_directory: examples_dir.clone(),
        default_extension: "mdc".to_string(),
        include_patterns: vec!["**/*.mdc".to_string()],
        exclude_patterns: vec![],
        watch_for_changes: false,
        recursion_depth: -1,
    };
    
    let manager = create_rule_directory_manager_with_root_dir(examples_dir.clone());
    manager.initialize().await?;
    
    // Get all rule files
    println!("Finding rule files...");
    let rule_files = manager.get_all_rule_files().await?;
    
    println!("Found {} rule files:\n", rule_files.len());
    
    for (i, path) in rule_files.iter().enumerate() {
        println!("{}: {}", i + 1, path.display());
        
        // Parse the rule file
        let rule = parse_rule_file(path).await?;
        
        println!("  - ID: {}", rule.id);
        println!("  - Name: {}", rule.name);
        println!("  - Description: {}", rule.description);
        println!("  - Category: {}", rule.category);
        println!("  - Patterns: {}", rule.patterns.join(", "));
        println!("  - Conditions: {}", rule.conditions.len());
        println!("  - Actions: {}", rule.actions.len());
        
        // Test if the rule matches some contexts
        let test_contexts = [
            "context.test", 
            "special.context", 
            "other.context"
        ];
        
        println!("  - Matching test:");
        for context_id in test_contexts {
            let matches = rule_matches_context(&rule, context_id);
            println!("    - {}: {}", context_id, if matches { "MATCH" } else { "NO MATCH" });
        }
        
        println!();
    }
    
    // Demonstrate creating a new rule
    println!("Creating a new rule template...");
    let template = create_rule_template("new-rule", "New Rule", "example");
    println!("{}\n", template);
    
    // Create the rule file
    println!("Creating the rule file...");
    let rule_path = manager.create_rule_file("new-rule", None::<String>, &template).await?;
    println!("Rule file created at: {}\n", rule_path.display());
    
    // Reload rules
    println!("Reloading rules...");
    let rule_files = manager.get_all_rule_files().await?;
    println!("Found {} rule files after adding the new rule.\n", rule_files.len());
    
    println!("Example completed successfully!");
    
    Ok(())
} 