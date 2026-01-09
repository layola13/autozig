//! 最小化测试示例 - 验证 #[autozig] 属性宏功能

use autozig::include_zig;
use wasm_bindgen::prelude::*;

// 简单测试：验证宏语法解析
include_zig!("src/test.zig", {
    // 测试1：基本双重绑定
    #[autozig(strategy = "dual")]
    fn add(a: i32, b: i32) -> i32;
    
    // 测试2：自定义前缀
    #[autozig(strategy = "dual", prefix_bindgen = "js_", prefix_c = "native_")]
    fn multiply(a: i32, b: i32) -> i32;
});

// 手动测试函数（对比）
#[wasm_bindgen]
pub fn manual_test() -> i32 {
    42
}