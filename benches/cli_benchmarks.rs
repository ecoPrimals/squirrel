use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;

// Import from squirrel_cli
use squirrel_cli::command_adapter::{CommandAdapter, CommandRegistry, RegistryAdapter};
use squirrel_cli::commands::test_command::TestCommand;

fn create_test_command(name: &str, description: &str, result: &str) -> Arc<dyn TestCommand> {
    Arc::new(TestCommand::new(name, description, result))
}

fn create_registry_with_commands() -> Arc<Mutex<CommandRegistry>> {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mut reg = registry.lock().await;
        // Register 10 test commands
        for i in 0..10 {
            reg.register(
                &format!("test{}", i),
                create_test_command(
                    &format!("test{}", i),
                    &format!("Test command {}", i),
                    &format!("Result {}", i),
                ),
            ).unwrap();
        }
    });

    registry
}

fn create_adapter() -> Arc<RegistryAdapter> {
    Arc::new(RegistryAdapter::new(create_registry_with_commands()))
}

fn benchmark_register_command(c: &mut Criterion) {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let rt = Runtime::new().unwrap();

    c.bench_function("register_command", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut reg = registry.lock().await;
                reg.register(
                    "benchmark_command",
                    create_test_command("benchmark_command", "Benchmark command", "Benchmark result"),
                ).unwrap();
            })
        })
    });
}

fn benchmark_execute_command(c: &mut Criterion) {
    let adapter = create_adapter();
    let rt = Runtime::new().unwrap();

    c.bench_function("execute_command", |b| {
        b.iter(|| {
            rt.block_on(async {
                adapter.execute_command("test0", vec![black_box("arg1".to_string())]).await.unwrap()
            })
        })
    });
}

fn benchmark_list_commands(c: &mut Criterion) {
    let adapter = create_adapter();
    let rt = Runtime::new().unwrap();

    c.bench_function("list_commands", |b| {
        b.iter(|| {
            rt.block_on(async {
                adapter.list_commands().await.unwrap()
            })
        })
    });
}

fn benchmark_concurrent_command_execution(c: &mut Criterion) {
    let adapter = create_adapter();
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_command_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];
                
                // Create 50 concurrent tasks
                for i in 0..50 {
                    let adapter_clone = adapter.clone();
                    let cmd = format!("test{}", i % 10);
                    
                    handles.push(tokio::spawn(async move {
                        adapter_clone.execute_command(&cmd, vec![]).await
                    }));
                }
                
                // Wait for all tasks to complete
                for handle in handles {
                    let _ = handle.await.unwrap();
                }
            })
        })
    });
}

fn benchmark_command_with_lock_contention(c: &mut Criterion) {
    let registry = create_registry_with_commands();
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    let rt = Runtime::new().unwrap();

    c.bench_function("command_with_lock_contention", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create a task that holds the lock for a while
                let registry_clone = registry.clone();
                let lock_holder = tokio::spawn(async move {
                    let reg = registry_clone.lock().await;
                    // Hold the lock for 10ms
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    drop(reg);
                });
                
                // Immediately try to execute a command (will have to wait for the lock)
                let adapter_clone = adapter.clone();
                let command_executor = tokio::spawn(async move {
                    adapter_clone.execute_command("test0", vec![]).await
                });
                
                // Wait for both tasks
                let _ = lock_holder.await;
                let _ = command_executor.await;
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_register_command,
    benchmark_execute_command,
    benchmark_list_commands,
    benchmark_concurrent_command_execution,
    benchmark_command_with_lock_contention
);
criterion_main!(benches); 