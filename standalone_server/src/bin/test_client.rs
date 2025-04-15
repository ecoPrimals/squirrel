use anyhow::Result;
use clap::Parser;
use taskserver_standalone::client::run_test_client;

#[derive(Parser, Debug)]
#[clap(name = "task_client", about = "Test client for the Task Management Server")]
struct Opt {
    /// Socket address of the task server
    #[clap(short, long, default_value = "http://[::1]:50052")]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let opt = Opt::parse();
    
    println!("Starting Task Client to test server at {}", opt.address);
    
    // Run client test
    run_test_client(&opt.address).await?;
    
    Ok(())
} 