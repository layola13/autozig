//! AutoZig WASM64 Print Example
//!
//! 展示如何在 WASM64 环境下使用 console.log 输出日志
//!
//! 核心特性：
//! - ✅ 支持 WASM64 (Memory64)
//! - ✅ BigInt 指针自动处理
//! - ✅ 零拷贝字符串传递
//! - ✅ Rust + Zig 混合编程
//! - ✅ 无需 wasm-bindgen

use autozig::{
    autozig_export,
    include_zig,
};

// 引入 Zig console 实现
include_zig!("src/console.zig", {
    fn console_log(ptr: *const u8, len: usize);
    fn console_error(ptr: *const u8, len: usize);
});

/// Rust 宏：console_log! - 格式化输出到控制台
macro_rules! console_log {
    ($($t:tt)*) => {
        {
            let s = format!($($t)*);
            console_log(s.as_ptr(), s.len());
        }
    }
}

/// Rust 宏：console_error! - 格式化错误输出到控制台
macro_rules! console_error {
    ($($t:tt)*) => {
        {
            let s = format!($($t)*);
            console_error(s.as_ptr(), s.len());
        }
    }
}

/// 初始化函数
#[autozig_export]
pub fn init() {
    console_log!("🚀 Initializing WASM64 Print Example...");

    // 设置 panic hook
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        console_error!("RUST PANIC: {}", msg);
    }));

    console_log!("✅ Panic hook installed");
    console_log!("✅ WASM64 Print Example ready!");
}

/// 测试函数1: 简单的加法
#[autozig_export]
pub fn add(a: i32, b: i32) -> i32 {
    console_log!("📝 add({}, {}) called", a, b);
    let result = a + b;
    console_log!("✅ add result: {}", result);
    result
}

/// 测试函数2: 计算阶乘
#[autozig_export]
pub fn factorial(n: u32) -> u64 {
    console_log!("📝 factorial({}) called", n);

    if n == 0 || n == 1 {
        console_log!("✅ factorial base case: 1");
        return 1;
    }

    let mut result = 1u64;
    for i in 2..=n {
        result *= i as u64;
        console_log!("  → step {}: {}", i, result);
    }

    console_log!("✅ factorial result: {}", result);
    result
}

/// 测试函数3: 字符串问候
#[autozig_export]
pub fn greet(name: &str) -> String {
    console_log!("📝 greet(\"{}\") called", name);
    let greeting = format!("Hello, {}! 🎉", name);
    console_log!("✅ greeting: {}", greeting);
    greeting
}

/// 测试函数4: 数组求和
#[autozig_export]
pub fn sum_array(data: &[i32]) -> i32 {
    console_log!("📝 sum_array called with {} elements", data.len());
    console_log!("  → data: {:?}", data);

    let sum: i32 = data.iter().sum();
    console_log!("✅ sum: {}", sum);
    sum
}

/// 测试函数5: 除法（演示错误处理）
#[autozig_export]
pub fn divide(a: i32, b: i32) -> i32 {
    console_log!("📝 divide({}, {}) called", a, b);

    if b == 0 {
        console_error!("❌ Error: Division by zero!");
        return 0;
    }

    let result = a / b;
    console_log!("✅ divide result: {}", result);
    result
}

/// 测试函数6: 触发 panic（演示 panic hook）
#[autozig_export]
pub fn test_panic() {
    console_log!("⚠️  About to trigger panic...");
    panic!("This is a test panic!");
}

/// 获取版本
#[autozig_export]
pub fn get_version() -> u32 {
    100 // v1.0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_greet() {
        let greeting = greet("AutoZig");
        assert!(greeting.contains("AutoZig"));
        assert!(greeting.contains("Hello"));
    }

    #[test]
    fn test_sum_array() {
        assert_eq!(sum_array(&[1, 2, 3, 4, 5]), 15);
        assert_eq!(sum_array(&[]), 0);
        assert_eq!(sum_array(&[-1, -2, -3]), -6);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), 5);
        assert_eq!(divide(10, 0), 0); // Error case
    }
}
