// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use std::env;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Once;
use std::path::PathBuf;

static DOCKER_START: Once = Once::new();
static DOCKER_STOP: Once = Once::new();

/// Get the project root directory
fn get_project_root() -> PathBuf {
    // Start from the current dir
    let mut path = env::current_dir().unwrap();
    
    // Go up until we find the docker-compose.yml file
    while !path.join("docker-compose.yml").exists() {
        if !path.pop() {
            // We've reached the root and still haven't found it
            // Just return the starting directory as a fallback
            return env::current_dir().unwrap();
        }
    }
    
    path
}

/// Start the required Docker services for tracing tests
pub async fn start_docker_services() -> bool {
    let mut success = false;
    
    DOCKER_START.call_once(|| {
        // Check if services are already running
        let status_check = Command::new("docker")
            .args(["ps", "--filter", "name=otel-collector", "--format", "{{.Names}}"])
            .output();
            
        match status_check {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    // Services are already running
                    println!("OpenTelemetry services are already running");
                    success = true;
                    return;
                }
            },
            Err(e) => {
                println!("Failed to check Docker status: {}", e);
                return;
            }
        }
        
        // Start the services
        println!("Starting OpenTelemetry services...");
        let project_root = get_project_root();
        println!("Using project root: {:?}", project_root);
        
        let start_result = Command::new("docker-compose")
            .args(["up", "-d"])
            .current_dir(&project_root)
            .output();
            
        match start_result {
            Ok(output) => {
                println!("Docker startup output: {}", String::from_utf8_lossy(&output.stdout));
                if !output.status.success() {
                    println!("Docker startup error: {}", String::from_utf8_lossy(&output.stderr));
                    return;
                }
                success = true;
            },
            Err(e) => {
                println!("Failed to start Docker services: {}", e);
                return;
            }
        }
    });
    
    // Wait for services to be ready if they were just started
    if success {
        println!("Waiting for services to be ready...");
        sleep(Duration::from_secs(5)).await;
    }
    
    success
}

/// Stop the Docker services
pub fn stop_docker_services() {
    DOCKER_STOP.call_once(|| {
        println!("Stopping OpenTelemetry services...");
        let project_root = get_project_root();
        
        let stop_result = Command::new("docker-compose")
            .args(["down"])
            .current_dir(&project_root)
            .output();
            
        match stop_result {
            Ok(output) => {
                println!("Docker stop output: {}", String::from_utf8_lossy(&output.stdout));
                if !output.status.success() {
                    println!("Docker stop error: {}", String::from_utf8_lossy(&output.stderr));
                }
            },
            Err(e) => {
                println!("Failed to stop Docker services: {}", e);
            }
        }
    });
}

/// Check if OpenTelemetry services are available for testing
pub async fn check_otel_services() -> bool {
    // Check OTLP HTTP endpoint
    let status = reqwest::Client::new()
        .get("http://localhost:4318/")
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    match status {
        Ok(_) => true,
        Err(e) => {
            println!("OpenTelemetry collector not available: {}", e);
            false
        }
    }
}

/// Check if Jaeger is available for testing
pub async fn check_jaeger_services() -> bool {
    // Check Jaeger UI endpoint
    let status = reqwest::Client::new()
        .get("http://localhost:16686/")
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    match status {
        Ok(_) => true,
        Err(e) => {
            println!("Jaeger not available: {}", e);
            false
        }
    }
}

/// Check if Zipkin is available for testing
pub async fn check_zipkin_services() -> bool {
    // Check Zipkin UI endpoint
    let status = reqwest::Client::new()
        .get("http://localhost:9411/")
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    match status {
        Ok(_) => true,
        Err(e) => {
            println!("Zipkin not available: {}", e);
            false
        }
    }
} 