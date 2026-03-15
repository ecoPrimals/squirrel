// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration tests for Universal Transport
//!
//! These tests validate the complete client-server transport stack
//! with actual connections on supported platforms.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use universal_patterns::transport::{
    ListenerConfig, TransportConfig, TransportType, UniversalListener, UniversalTransport,
};

/// Helper function to get a unique test service name
fn test_service_name(test_name: &str) -> String {
    format!("test_{}_{}", test_name, std::process::id())
}

/// Test basic TCP client-server connection
///
/// This test validates that the universal transport can establish
/// a TCP connection and transfer data bidirectionally.
#[tokio::test]
async fn test_tcp_echo_server() {
    let service_name = test_service_name("tcp_echo");

    // Configure both sides to use TCP explicitly
    let mut listener_config = ListenerConfig::default();
    listener_config.preferred_transport = Some(TransportType::Tcp);
    listener_config.enable_fallback = false;

    let mut client_config = TransportConfig::default();
    client_config.preferred_transport = Some(TransportType::Tcp);
    client_config.enable_fallback = false;

    // Start server
    let listener = UniversalListener::bind(&service_name, Some(listener_config))
        .await
        .expect("Failed to bind server");

    println!("Server bound to: {}", listener.local_addr().unwrap());

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let (mut stream, addr) = listener.accept().await.expect("Failed to accept connection");
        println!("Server accepted connection from {:?}", addr);

        // Echo server: read and write back
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await.expect("Failed to read");
        stream
            .write_all(&buf[..n])
            .await
            .expect("Failed to write");
        println!("Server echoed {} bytes", n);
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let mut client = UniversalTransport::connect(&service_name, Some(client_config))
        .await
        .expect("Failed to connect");

    println!("Client connected");

    // Send data
    let test_data = b"Hello, Universal Transport!";
    client
        .write_all(test_data)
        .await
        .expect("Failed to send data");
    println!("Client sent {} bytes", test_data.len());

    // Receive echo
    let mut buf = vec![0; 1024];
    let n = client.read(&mut buf).await.expect("Failed to read echo");
    println!("Client received {} bytes", n);

    assert_eq!(&buf[..n], test_data, "Echo data should match sent data");

    // Wait for server to finish
    server_handle.await.expect("Server task failed");

    println!("✅ TCP echo test passed!");
}

/// Test Unix socket client-server connection (Linux/macOS only)
///
/// This test validates Unix domain socket connections with the
/// universal transport abstraction.
#[tokio::test]
#[cfg(unix)]
async fn test_unix_socket_echo_server() {
    let service_name = test_service_name("unix_echo");

    // Configure both sides to use Unix filesystem sockets
    let mut listener_config = ListenerConfig::default();
    listener_config.preferred_transport = Some(TransportType::UnixFilesystem);
    listener_config.enable_fallback = false;

    let mut client_config = TransportConfig::default();
    client_config.preferred_transport = Some(TransportType::UnixFilesystem);
    client_config.enable_fallback = false;

    // Start server
    let listener = UniversalListener::bind(&service_name, Some(listener_config))
        .await
        .expect("Failed to bind Unix socket server");

    println!("Unix socket server bound to: {}", listener.local_addr().unwrap());

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let (mut stream, addr) = listener.accept().await.expect("Failed to accept connection");
        println!("Server accepted Unix socket connection from {:?}", addr);

        // Echo server
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await.expect("Failed to read");
        stream
            .write_all(&buf[..n])
            .await
            .expect("Failed to write");
        println!("Server echoed {} bytes", n);
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let mut client = UniversalTransport::connect(&service_name, Some(client_config))
        .await
        .expect("Failed to connect to Unix socket");

    println!("Client connected via Unix socket");

    // Send data
    let test_data = b"Unix socket test data!";
    client
        .write_all(test_data)
        .await
        .expect("Failed to send data");

    // Receive echo
    let mut buf = vec![0; 1024];
    let n = client.read(&mut buf).await.expect("Failed to read echo");

    assert_eq!(&buf[..n], test_data, "Echo data should match sent data");

    // Wait for server to finish
    server_handle.await.expect("Server task failed");

    println!("✅ Unix socket echo test passed!");
}

/// Test automatic fallback to TCP when preferred transport fails
///
/// This test validates that the universal transport automatically
/// falls back to TCP when the preferred transport is unavailable.
#[tokio::test]
async fn test_automatic_fallback_to_tcp() {
    let service_name = test_service_name("fallback_tcp");

    // Server: Bind with TCP fallback enabled (default)
    let listener = UniversalListener::bind(&service_name, None)
        .await
        .expect("Failed to bind server with fallback");

    println!("Server bound (with fallback) to: {}", listener.local_addr().unwrap());

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let (mut stream, addr) = listener.accept().await.expect("Failed to accept connection");
        println!("Server accepted connection from {:?}", addr);

        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await.expect("Failed to read");
        stream
            .write_all(&buf[..n])
            .await
            .expect("Failed to write");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Client: Connect with fallback enabled (should work even if Unix socket fails)
    let mut client = UniversalTransport::connect(&service_name, None)
        .await
        .expect("Failed to connect with fallback");

    println!("Client connected via fallback transport");

    // Test communication
    let test_data = b"Fallback test data!";
    client.write_all(test_data).await.expect("Failed to send");

    let mut buf = vec![0; 1024];
    let n = client.read(&mut buf).await.expect("Failed to read");

    assert_eq!(&buf[..n], test_data);

    server_handle.await.expect("Server task failed");

    println!("✅ Automatic fallback test passed!");
}

/// Test concurrent connections to the same server
///
/// This test validates that the universal transport can handle
/// multiple concurrent client connections.
#[tokio::test]
async fn test_concurrent_connections() {
    let service_name = test_service_name("concurrent");

    // Configure to use TCP for reliable concurrent testing
    let mut listener_config = ListenerConfig::default();
    listener_config.preferred_transport = Some(TransportType::Tcp);

    let listener = UniversalListener::bind(&service_name, Some(listener_config))
        .await
        .expect("Failed to bind server");

    println!("Concurrent test server bound to: {}", listener.local_addr().unwrap());

    // Spawn server task that accepts multiple connections
    let server_handle = tokio::spawn(async move {
        for i in 0..3 {
            let (mut stream, addr) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            println!("Server accepted connection {} from {:?}", i, addr);

            // Spawn handler for each connection
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                let n = stream.read(&mut buf).await.expect("Failed to read");
                stream
                    .write_all(&buf[..n])
                    .await
                    .expect("Failed to write");
            });
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn multiple clients concurrently
    let mut client_handles = vec![];

    for i in 0..3 {
        let service_name = service_name.clone();
        let handle = tokio::spawn(async move {
            let mut client_config = TransportConfig::default();
            client_config.preferred_transport = Some(TransportType::Tcp);

            let mut client = UniversalTransport::connect(&service_name, Some(client_config))
                .await
                .expect("Failed to connect");

            println!("Client {} connected", i);

            let test_data = format!("Client {} data", i);
            client
                .write_all(test_data.as_bytes())
                .await
                .expect("Failed to send");

            let mut buf = vec![0; 1024];
            let n = client.read(&mut buf).await.expect("Failed to read");

            assert_eq!(&buf[..n], test_data.as_bytes());
            println!("Client {} verified echo", i);
        });

        client_handles.push(handle);
    }

    // Wait for all clients to finish
    for handle in client_handles {
        handle.await.expect("Client task failed");
    }

    // Wait for server to finish
    server_handle.await.expect("Server task failed");

    println!("✅ Concurrent connections test passed!");
}

/// Test large data transfer
///
/// This test validates that the universal transport can handle
/// larger data transfers correctly.
#[tokio::test]
async fn test_large_data_transfer() {
    let service_name = test_service_name("large_data");

    let mut listener_config = ListenerConfig::default();
    listener_config.preferred_transport = Some(TransportType::Tcp);

    let listener = UniversalListener::bind(&service_name, Some(listener_config))
        .await
        .expect("Failed to bind server");

    println!("Large data test server bound");

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let (mut stream, _addr) = listener.accept().await.expect("Failed to accept");

        // Read all data
        let mut received = Vec::new();
        let mut buf = vec![0; 8192];

        loop {
            match stream.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    received.extend_from_slice(&buf[..n]);
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }

        println!("Server received {} bytes total", received.len());

        // Echo back
        stream
            .write_all(&received)
            .await
            .expect("Failed to write");
        stream.shutdown().await.expect("Failed to shutdown");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let mut client_config = TransportConfig::default();
    client_config.preferred_transport = Some(TransportType::Tcp);

    let mut client = UniversalTransport::connect(&service_name, Some(client_config))
        .await
        .expect("Failed to connect");

    // Generate large test data (1 MB)
    let test_data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
    println!("Client sending {} bytes", test_data.len());

    // Send data
    client
        .write_all(&test_data)
        .await
        .expect("Failed to send data");
    client.shutdown().await.expect("Failed to shutdown write");

    // Receive echo
    let mut received = Vec::new();
    let mut buf = vec![0; 8192];

    loop {
        match client.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                received.extend_from_slice(&buf[..n]);
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    println!("Client received {} bytes", received.len());

    assert_eq!(
        received.len(),
        test_data.len(),
        "Received data length should match sent data"
    );
    assert_eq!(received, test_data, "Received data should match sent data");

    server_handle.await.expect("Server task failed");

    println!("✅ Large data transfer test passed!");
}

/// Test connection timeout
///
/// This test validates that connection attempts timeout appropriately
/// when the server is not available.
#[tokio::test]
async fn test_connection_timeout() {
    let service_name = test_service_name("timeout");

    // Configure very short timeout
    let mut client_config = TransportConfig::default();
    client_config.preferred_transport = Some(TransportType::Tcp);
    client_config.timeout_ms = 100; // 100ms timeout
    client_config.enable_fallback = false; // Don't fallback, we want timeout

    // Try to connect to non-existent server (should timeout)
    let result = UniversalTransport::connect(&service_name, Some(client_config)).await;

    assert!(
        result.is_err(),
        "Connection should fail when server is not available"
    );

    if let Err(e) = result {
        println!("Expected connection failure: {}", e);
    }

    println!("✅ Connection timeout test passed!");
}

/// Test transport type detection
///
/// This test validates that we can query the transport type
/// being used by a connection.
#[tokio::test]
async fn test_transport_type_detection() {
    let service_name = test_service_name("type_detect");

    // Bind server with TCP
    let mut listener_config = ListenerConfig::default();
    listener_config.preferred_transport = Some(TransportType::Tcp);
    listener_config.enable_fallback = false;

    let listener = UniversalListener::bind(&service_name, Some(listener_config))
        .await
        .expect("Failed to bind server");

    // Spawn server
    let _server_handle = tokio::spawn(async move {
        let (_stream, _addr) = listener.accept().await.expect("Failed to accept");
        // Just accept and hold
        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let mut client_config = TransportConfig::default();
    client_config.preferred_transport = Some(TransportType::Tcp);
    client_config.enable_fallback = false;

    let transport = UniversalTransport::connect(&service_name, Some(client_config))
        .await
        .expect("Failed to connect");

    // Verify transport type
    assert_eq!(
        transport.transport_type(),
        TransportType::Tcp,
        "Transport should be TCP"
    );

    println!("✅ Transport type detection test passed!");
}
