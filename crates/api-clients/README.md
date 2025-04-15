# Squirrel API Clients

This crate provides a collection of clients for interacting with external APIs that the Squirrel system integrates with. It handles authentication, rate limiting, request building, and response parsing in a consistent way.

## Features

- Common HTTP client interface with consistent error handling
- Authentication mechanisms (Basic, Bearer, OAuth2)
- Rate limiting and retry mechanisms
- Request and response middleware
- Typed API clients for popular services

## Usage

### Basic HTTP Client

```rust
use squirrel_api_clients::http::{HttpClientConfig, default_client};
use squirrel_api_clients::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client with default settings
    let client = default_client();
    
    // Make a request
    let response: serde_json::Value = client.get_json("https://api.example.com/data").await?;
    
    println!("Response: {:?}", response);
    
    Ok(())
}
```

### Custom HTTP Client

```rust
use squirrel_api_clients::http::{HttpClientConfig, new_client};
use squirrel_api_clients::auth;
use squirrel_api_clients::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a bearer token authenticator
    let auth = auth::bearer_auth("your-api-token");
    
    // Configure the client
    let config = HttpClientConfig::new("https://api.example.com")
        .with_timeout(60)
        .with_header("User-Agent", "Squirrel/1.0")
        .with_rate_limit(100); // 100 requests per minute
    
    // Create a client with authentication
    let client = new_client(config, Some(auth));
    
    // Make an authenticated request
    let response: serde_json::Value = client.get_json("/data").await?;
    
    println!("Response: {:?}", response);
    
    Ok(())
}
```

### GitHub API Client

```rust
use squirrel_api_clients::github::GitHubClient;
use squirrel_api_clients::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a GitHub client with a personal access token
    let github = GitHubClient::with_token("your-github-token");
    
    // Get information about the authenticated user
    let user = github.get_authenticated_user().await?;
    println!("Logged in as: {}", user.login);
    
    // List repositories
    let repos = github.list_repos().await?;
    println!("You have {} repositories", repos.len());
    
    // Get a specific repository
    let repo = github.get_repo("owner", "repo").await?;
    println!("Repo: {}", repo.full_name);
    
    Ok(())
}
```

## Available API Clients

- GitHub - Basic implementation available
- HTTP - General purpose HTTP client
- More coming soon!

## Adding New API Clients

To add a new API client:

1. Create a new module in `src/<service_name>/`
2. Define data models specific to the API
3. Implement a client that uses the common HTTP client interface
4. Add any specialized methods needed for the API

## License

This crate is part of the Squirrel project. 