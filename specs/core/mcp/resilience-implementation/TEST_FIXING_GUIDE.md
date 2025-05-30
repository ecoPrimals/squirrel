# Resilience Module Test Fixing Guide

## Common Issues and Solutions

This guide provides detailed instructions for fixing the most common test failures in the resilience module.

## 1. Method Visibility Issues

### Problem: Private `calculate_delay` Method

```
error[E0624]: method `calculate_delay` is private
--> crates\mcp\src\resilience\tests\retry_tests.rs:116:22
    |
116 |     assert_eq!(retry.calculate_delay(3).as_millis(), 30);
    |                      ^^^^^^^^^^^^^^^ private method
```

### Solution:

Change the visibility of the `calculate_delay` method in `retry.rs`:

```rust
// Before
fn calculate_delay(&self, attempt: u32) -> Duration {
    // implementation
}

// After
pub fn calculate_delay(&self, attempt: u32) -> Duration {
    // implementation
}
```

## 2. Async Test Issues

### Problem: Missing `.await` on Futures

```
error[E0599]: no method named `is_ok` found for opaque type `impl Future<Output = Result<(), StateSyncError>>` in the current scope
--> crates\mcp\src\resilience\tests\state_sync_tests.rs:36:20
    |
36  |     assert!(result.is_ok());
    |                    ^^^^^
```

### Solution:

Add `.await` to properly resolve the Future before calling methods on the Result:

```rust
// Before
assert!(result.is_ok());

// After
assert!(result.await.is_ok());
```

Apply this pattern to all similar issues:
- `result.is_ok()` → `result.await.is_ok()`
- `result.is_err()` → `result.await.is_err()`
- `result.unwrap()` → `result.await.unwrap()`

## 3. Owned vs. Reference Issues

### Problem: Passing Reference to Function Expecting Owned Value

```
error[E0308]: mismatched types
--> crates\mcp\src\resilience\tests\integration_tests.rs:57:13
    |
57  |             &retry,
    |             ^^^^^^ expected `RetryMechanism`, found `&RetryMechanism`
```

### Solution:

Remove the borrowing operator when passing the RetryMechanism:

```rust
// Before
let result = with_resilience(
    &mut circuit_breaker,
    &retry,
    // other parameters
);

// After
let result = with_resilience(
    &mut circuit_breaker,
    retry,  // Remove the & operator
    // other parameters
);
```

When removal of the reference would consume the RetryMechanism that's needed later, use clone():

```rust
let result = with_resilience(
    &mut circuit_breaker,
    retry.clone(),
    // other parameters
);
```

## 4. Type Annotation Issues

### Problem: Missing Type Annotations in Recovery Tests

```
error[E0282]: type annotations needed for `std::result::Result<_, resilience::recovery::RecoveryError>`
--> crates\mcp\src\resilience\tests\recovery_tests.rs:143:9
```

### Solution:

Add explicit type annotations to help the compiler infer the correct types:

```rust
// Before
let result = recovery.handle_failure(failure, || {
    // function body
});

// After
let result: std::result::Result<(), resilience::recovery::RecoveryError> = 
    recovery.handle_failure(failure, || {
        // function body
    });
```

## 5. Method Name Changes

### Problem: `get_state()` vs `state()` Method Name

```
error[E0599]: no method named `get_state` found for struct `CircuitBreaker` in the current scope
--> crates\mcp\src\resilience\tests\integration_tests.rs:94:32
```

### Solution:

Update the method calls to use the correct method name:

```rust
// Before
assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

// After
assert_eq!(circuit_breaker.state(), CircuitState::Open);
```

## 6. Return Type Mismatches

### Problem: Returning Immediate Result vs. Boxed Future

```
error[E0308]: mismatched types
--> crates\mcp\src\resilience\tests\integration_tests.rs:140:17
    |
140 |     Err(Box::new(TestError("Temporary error".to_string())))
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Pin<Box<...>>`, found `Result<_, Box<TestError>>`
```

### Solution:

Wrap the result in a boxed Future:

```rust
// Before
operation = || {
    Err(Box::new(TestError("Temporary error".to_string())))
};

// After
operation = || {
    Box::pin(async {
        Err(Box::new(TestError("Temporary error".to_string())))
    })
};
```

## 7. Error Type Coercion

### Problem: Error type mismatches

```
error[E0271]: expected `{closure@integration_tests.rs:357:35}` to be a closure that returns `Result<_, Box<dyn Error + Send + Sync>>`, but it returns `Result<_, Box<TestError>>`
```

### Solution:

Correctly coerce error types by using dyn Error:

```rust
// Before
Err(Box::new(TestError("Error message".to_string())))

// After
Err(Box::new(TestError("Error message".to_string())) as Box<dyn std::error::Error + Send + Sync>)
```

## Example Complete Test Fix

Here's an example of a fully fixed test case that addresses multiple issues:

```rust
#[tokio::test]
async fn test_retry_with_circuit_breaker() {
    // Setup
    let mut circuit_breaker = CircuitBreaker::default();
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
        ..RetryConfig::default()
    };
    let retry = RetryMechanism::new(retry_config);
    
    // Test successful operation
    let result = with_resilience(
        &mut circuit_breaker,
        retry.clone(), // Use clone() instead of reference
        || {
            Box::pin(async {
                // Return a boxed future
                Ok("Success".to_string())
            })
        }
    );
    
    // Properly await the result
    assert!(result.await.is_ok());
    assert_eq!(result.await.unwrap(), "Success".to_string());
    
    // Check the circuit breaker state with correct method name
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}
```

## Implementation Checklist

As you fix each test file, work through these steps:

1. Fix method visibility issues in the source code
2. Update async test assertions to use `.await`
3. Fix owned vs. reference issues by removing `&` where needed
4. Add explicit type annotations where needed
5. Update method names to match the current API
6. Fix return type mismatches by properly boxing futures
7. Correct error type coercions 

Remember to run the tests after each significant change to verify your fixes. 