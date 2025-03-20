use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Web client test program starting...");
    let client = Client::new();
    
    // Test health endpoint
    println!("Testing health endpoint...");
    let response = client.get("http://localhost:3000/api/health")
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    let body: Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&body)?);
    
    // Test creating a job
    println!("\nTesting job creation...");
    let response = client.post("http://localhost:3000/api/jobs")
        .json(&serde_json::json!({
            "name": "test-job",
            "parameters": {
                "param1": "value1",
                "param2": 42
            }
        }))
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    let body: Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&body)?);
    
    // Get job status using the URL from the previous response
    let job_id = body["job_id"].as_str().unwrap();
    println!("\nTesting job status retrieval for job {}...", job_id);
    let response = client.get(format!("http://localhost:3000/api/jobs/{}", job_id))
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    let body: Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&body)?);
    
    // Test job report endpoint
    println!("\nTesting job report for job {}...", job_id);
    let response = client.get(format!("http://localhost:3000/api/jobs/{}/report", job_id))
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    let body: Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&body)?);
    
    println!("\nAll tests completed successfully!");
    Ok(())
} 