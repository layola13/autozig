//! AutoZig Rust Export Example
//! 
//! 展示如何使用 #[autozig_export] 宏直接从 Rust 导出函数到 WASM
//! 无需 Zig wrapper 或 wasm-bindgen

use autozig::autozig_export;

/// 简单的加法函数
#[autozig_export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 乘法函数
#[autozig_export]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// 获取版本号
#[autozig_export]
pub fn get_version() -> u32 {
    100 // v1.0.0
}

/// 计算平方
#[autozig_export]
pub fn square(x: i32) -> i32 {
    x * x
}

/// 判断是否为偶数
#[autozig_export]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
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
        assert_eq!(multiply(4, 5), 20);
    }
    
    #[test]
    fn test_square() {
        assert_eq!(square(7), 49);
    }
    
    #[test]
    fn test_is_even() {
        assert!(is_even(4));
        assert!(!is_even(5));
    }
}