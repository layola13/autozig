const std = @import("std");

/// Common Exchange Layout
/// Must match Rust's ZigBuffer layout exactly.
pub const ZigBuffer = extern struct {
    ptr: [*]u8,
    len: usize,
    cap: usize,
    /// Function pointer to free the memory.
    /// Signature: fn(ptr: [*]u8, len: usize, cap: usize) void
    free_fn: ?*const fn ([*]u8, usize, usize) callconv(.c) void,
};

/// Standard free wrapper for C allocator
pub export fn zig_free_c(ptr: [*]u8, len: usize, cap: usize) void {
    _ = len;
    // Reconstruction of the slice to free it
    const slice = ptr[0..cap];
    allocator.free(slice);
}

/// Helper to wrap a Zig slice into a ZigBuffer for Rust
/// The caller must ensure the slice was allocated with c_allocator if using zig_free_c,
/// or provide a custom free function.
pub fn into_zig_buffer(slice: []u8) ZigBuffer {
    return ZigBuffer{
        .ptr = slice.ptr,
        .len = slice.len,
        .cap = slice.len, // Assuming len == cap for simplicity in this helper, but ideally should be cap
        .free_fn = zig_free_c,
    };
}

// Global allocator for this example
const allocator = std.heap.c_allocator;

// Exported function to allocate memory and give it to Rust
export fn allocate_data(size: usize) ZigBuffer {
    const slice = allocator.alloc(u8, size) catch unreachable;
    // Fill with some data
    @memset(slice, 0xAA);

    return ZigBuffer{
        .ptr = slice.ptr,
        .len = slice.len,
        .cap = slice.len,
        .free_fn = zig_free_c,
    };
}

/// Consumes the buffer (takes ownership) and frees it using the provided callback.
export fn take_ownership(buf: ZigBuffer) void {
    // We own buf now.
    // Simulate usage...
    if (buf.len > 0) {
        buf.ptr[0] = 0xFF;
    }

    // Free it using the callback provided by Rust (or whoever created it)
    if (buf.free_fn) |free_fn| {
        free_fn(buf.ptr, buf.len, buf.cap);
    }
}
