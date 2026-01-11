use std::time::Instant;

use autozig::{
    ffi_types::{
        ZigBox,
        ZigBuffer,
    },
    prelude::*,
};

// Use the memory monitoring provided by the OS
#[cfg(target_os = "linux")]
fn get_memory_usage() -> usize {
    let output = std::fs::read_to_string("/proc/self/statm").expect("Failed to read statm");
    let parts: Vec<&str> = output.split_whitespace().collect();
    let rss_pages: usize = parts[1].parse().expect("Failed to parse RSS");
    let page_size = 4096;
    rss_pages * page_size
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> usize {
    0
}

// ============================================================================
// 🛡️ Safe Interface Layer (The Bridge)
// ============================================================================
mod zig_api {
    use super::*;

    // Nest the raw FFI in a submodule to avoid name collisions
    mod raw_ffi {
        use super::*;

        autozig! {
            const std = @import("std");

            pub const ZigBuffer = extern struct {
                ptr: [*]u8,
                len: usize,
                cap: usize,
                free_fn: ?*const fn ([*]u8, usize, usize) callconv(.c) void,
            };

            pub export fn zig_free_c(ptr: [*]u8, len: usize, cap: usize) void {
                _ = len;
                const slice = ptr[0..cap];
                allocator.free(slice);
            }

            const allocator = std.heap.c_allocator;

            export fn allocate_data(size: usize) ZigBuffer {
                const slice = allocator.alloc(u8, size) catch unreachable;
                @memset(slice, 0xAA);
                return ZigBuffer{
                    .ptr = slice.ptr,
                    .len = slice.len,
                    .cap = slice.len,
                    .free_fn = zig_free_c,
                };
            }

            export fn take_ownership(buf: ZigBuffer) void {
                if (buf.len > 0) {
                    buf.ptr[0] = 0xFF;
                }
                if (buf.free_fn) |free_fn| {
                    free_fn(buf.ptr, buf.len, buf.cap);
                }
            }

            ---
            // Raw FFI signatures
            fn allocate_data(size: usize) -> ZigBuffer;
            fn take_ownership(buf: ZigBuffer);
        }
    }

    // ------------------------------------------------------------------------
    // Safe Public API (Zero Unsafe for Consumers)
    // ------------------------------------------------------------------------

    /// Returns a ZigBox<u8> which automatically frees memory on drop.
    pub fn allocate_data(size: usize) -> ZigBox<u8> {
        // Safe because we trust our Zig implementation's protocol
        // Call the raw FFI function from the inner module
        let raw = raw_ffi::allocate_data(size);
        ZigBox::new(raw)
    }

    /// Transfers ownership of a ZigBuffer to Zig.
    pub fn take_ownership(buf: ZigBuffer) {
        // Call the raw FFI function from the inner module
        raw_ffi::take_ownership(buf);
    }
}

// ============================================================================
// 🧪 User Business Logic (Zero Unsafe!)
// ============================================================================

fn run_iteration() {
    // 1. Zig Allocation -> Rust Drop
    // 🛡️ Completely Safe!
    let _zig_data = zig_api::allocate_data(1024);
    // _zig_data drops -> calls Zig free -> No Leak

    // 2. Rust Allocation -> Zig Drop
    let mut data = Vec::with_capacity(1024);
    data.resize(1024, 0xBB);

    // 🛡️ Completely Safe!
    zig_api::take_ownership(ZigBuffer::from(data));
}

fn main() {
    println!("Starting Library-Powered Safe Bridge Leak Test (1,000,000 iterations)...");

    let start_mem = get_memory_usage();
    let start_time = Instant::now();

    for i in 0..1_000_000 {
        run_iteration();
        if i % 100_000 == 0 {
            use std::io::Write;
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }

    let duration = start_time.elapsed();
    let end_mem = get_memory_usage();
    let diff = if end_mem > start_mem {
        end_mem - start_mem
    } else {
        0
    };

    println!("\nDone in {:.2?}", duration);
    println!("Diff: {} bytes", diff);

    if diff < 1024 * 1024 {
        println!("✅ SUCCESS: No significant leak detected.");
    } else {
        println!("❌ FAILURE: Memory leaked!");
        std::process::exit(1);
    }
}
