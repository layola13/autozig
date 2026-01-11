//! Console logging implementation for WASM64
//!
//! This module provides console.log and console.error functionality
//! for WebAssembly 64-bit environments.
//!
//! The JavaScript host must provide:
//! - js_log(ptr: [*]const u8, len: usize)
//! - js_error(ptr: [*]const u8, len: usize)

const std = @import("std");

// Import functions from JavaScript environment
extern "env" fn js_log(ptr: [*]const u8, len: usize) void;
extern "env" fn js_error(ptr: [*]const u8, len: usize) void;

/// Log a message to console (wrapper for Rust)
export fn console_log(ptr: [*]const u8, len: usize) void {
    js_log(ptr, len);
}

/// Log an error to console (wrapper for Rust)
export fn console_error(ptr: [*]const u8, len: usize) void {
    js_error(ptr, len);
}

/// Panic handler that forwards to console.error
pub fn panic(msg: []const u8, _: ?*std.builtin.StackTrace, _: ?usize) noreturn {
    js_error(msg.ptr, msg.len);
    while (true) {}
}
