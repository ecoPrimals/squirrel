/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Divide two numbers
pub fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 {
        return Err("Division by zero");
    }
    Ok(a / b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
    }
    
    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 2), 3);
    }
    
    #[test]
    fn test_divide() {
        assert_eq!(divide(6, 2), Ok(3));
        assert_eq!(divide(1, 0), Err("Division by zero"));
    }
} 