use criterion::{black_box, criterion_group, criterion_main, Criterion};
use groundhog_mcp::{
    ConnectionManager, ConnectionConfig, MessageHandler, MessageHandlerConfig,
    PortManager, SecurityManager, ErrorHandler, StateManager, ContextManager,
};
use chrono::Duration;

fn connection_manager_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_manager");
    
    let config = ConnectionConfig {
        max_connections: 100,
        connection_timeout: Duration::seconds(30),
        keep_alive_interval: Duration::seconds(10),
        max_message_queue: 1000,
    };

    let port_manager = PortManager::new(8000, 9000).unwrap();
    let security_manager = SecurityManager::new();
    let message_handler = MessageHandler::new(
        MessageHandlerConfig {
            max_message_size: 1024,
            message_timeout: Duration::seconds(30),
            retry_attempts: 3,
            retry_delay: Duration::seconds(1),
        },
        security_manager.clone(),
        ErrorHandler::new(),
        StateManager::new(),
        ContextManager::new(),
    );
    let error_handler = ErrorHandler::new();

    let manager = ConnectionManager::new(
        config,
        port_manager,
        security_manager,
        message_handler,
        error_handler,
    );

    group.bench_function("new_connection", |b| {
        b.iter(|| {
            black_box(&manager);
        })
    });

    group.finish();
}

criterion_group!(benches, connection_manager_benchmark);
criterion_main!(benches); 