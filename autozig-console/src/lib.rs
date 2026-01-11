//! # AutoZig Console
//!
//! Console logging support for AutoZig WASM applications.
//!
//! This crate provides `console_log!` and `console_error!` macros that work in
//! WebAssembly environments (both WASM32 and WASM64), solving the problem of
//! Rust's standard `print!` and `println!` macros being ineffective in
//! browsers.
//!
//! ## Features
//!
//! - ✅ **Rust → Zig → JS** three-layer architecture
//! - ✅ **WASM64 BigInt** pointer support (64-bit addressing)
//! - ✅ **Zero-copy** string passing (direct memory access)
//! - ✅ **Automatic panic hook** integration
//! - ✅ **Type-safe** FFI (no unsafe code needed by users)
//! - ✅ **No wasm-bindgen** required
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! autozig-console = "0.1"
//! ```
//!
//! Then use it in your code:
//!
//! ```rust,no_run
//! use autozig_console::{
//!     console_error,
//!     console_log,
//!     init_panic_hook,
//! };
//!
//! #[no_mangle]
//! pub extern "C" fn main() {
//!     // Initialize panic hook (optional but recommended)
//!     init_panic_hook();
//!
//!     // Use console_log just like println!
//!     console_log!("Hello from WASM!");
//!     console_log!("Value: {}", 42);
//!     console_log!("Data: {:?}", vec![1, 2, 3]);
//!
//!     // Use console_error for errors
//!     console_error!("Error: Something went wrong!");
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │  Rust (User Code)               │
//! │  console_log!("Hello {}", name) │
//! └────────────┬────────────────────┘
//!              │ FFI call
//!              ↓
//! ┌─────────────────────────────────┐
//! │  Zig (Middle Layer)             │
//! │  export fn autozig_log_impl()   │
//! └────────────┬────────────────────┘
//!              │ extern "env"
//!              ↓
//! ┌─────────────────────────────────┐
//! │  JavaScript (Browser)           │
//! │  js_log(ptr, len)               │
//! │  console.log(text)              │
//! └─────────────────────────────────┘
//! ```

use autozig::autozig;

autozig! {
    // ==========================================
    // Zig Implementation (嵌入式 Zig 代码)
    // ==========================================

    // 1. 导入 JS 环境提供的函数
    //    注意：WASM64 下 usize 是 64位，对应 JS 的 BigInt
    extern "env" fn js_log(ptr: [*]const u8, len: usize) void;
    extern "env" fn js_error(ptr: [*]const u8, len: usize) void;

    // 2. 导出给 Rust 调用的包装函数
    export fn autozig_log_impl(ptr: [*]const u8, len: usize) void {
        js_log(ptr, len);
    }

    export fn autozig_error_impl(ptr: [*]const u8, len: usize) void {
        js_error(ptr, len);
    }

    // 3. Zig Panic Handler（可选：接管 Zig 的 panic）
    pub fn panic(msg: []const u8, _: ?*std.builtin.StackTrace, _: ?usize) noreturn {
        js_error(msg.ptr, msg.len);
        while (true) {}
    }

    ---

    // ==========================================
    // Rust Signatures (自动生成的绑定)
    // ==========================================
    fn autozig_log_impl(msg: &str);
    fn autozig_error_impl(msg: &str);
}

// ==========================================
// Public API - Macros
// ==========================================

/// Output a log message to the browser console.
///
/// This macro works like `println!` but outputs to the browser's JavaScript
/// console instead of stdout (which doesn't exist in WASM environments).
///
/// # Examples
///
/// ```rust,no_run
/// use autozig_console::console_log;
///
/// console_log!("Hello from WASM!");
/// console_log!("Value: {}", 42);
/// console_log!("Data: {:?}", vec![1, 2, 3]);
/// ```
///
/// # Browser Output
///
/// ```text
/// [AutoZig] Hello from WASM!
/// [AutoZig] Value: 42
/// [AutoZig] Data: [1, 2, 3]
/// ```
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        {
            let s = format!($($t)*);
            // AutoZig 自动处理 &str -> (ptr, len) 的转换
            $crate::autozig_log_impl(&s);
        }
    }
}

/// Output an error message to the browser console.
///
/// This macro works like `eprintln!` but outputs to the browser's JavaScript
/// console using `console.error()` instead of stderr.
///
/// # Examples
///
/// ```rust,no_run
/// use autozig_console::console_error;
///
/// console_error!("Error occurred!");
/// console_error!("Failed with code: {}", 404);
/// ```
///
/// # Browser Output
///
/// ```text
/// [AutoZig Error] Error occurred!
/// [AutoZig Error] Failed with code: 404
/// ```
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        {
            let s = format!($($t)*);
            $crate::autozig_error_impl(&s);
        }
    }
}

// ==========================================
// Public API - Functions
// ==========================================

/// Initialize panic hook to forward Rust panics to `console.error`.
///
/// Call this function once when your WASM module is initialized to ensure
/// that any Rust panics are properly logged to the browser console.
///
/// # Examples
///
/// ```rust,no_run
/// use autozig_console::init_panic_hook;
///
/// #[no_mangle]
/// pub extern "C" fn init() {
///     init_panic_hook();
///     // Now all panics will be logged to console.error
/// }
/// ```
///
/// # Panic Output
///
/// When a panic occurs:
///
/// ```text
/// [AutoZig Error] RUST PANIC: panicked at 'index out of bounds: ...'
/// ```
pub fn init_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        console_error!("RUST PANIC: {}", msg);
    }));
}

#[cfg(test)]
mod tests {
    // Note: Tests for WASM-only code cannot run on native targets
    // This is expected behavior - the Zig code only compiles for WASM targets
}
