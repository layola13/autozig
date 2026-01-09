# FFI Allocator Example

This example demonstrates the **correct pattern** for memory allocation in Zig FFI functions.

## Problem

When calling Zig FFI functions that allocate memory from Rust, using `?*T` (optional pointer) in Zig with `Option<*mut T>` in Rust causes **ABI incompatibility** issues, resulting in allocation failures.

## Solution

| Aspect | ❌ Wrong | ✅ Correct |
|--------|----------|-----------|
| Zig Allocator | Cross-file import | Local `GeneralPurposeAllocator` |
| Zig Return | `?*T` + `catch return null` | `*T` + `catch unreachable` |
| Rust FFI | `Option<*mut T>` | `*mut T` |

## Key Files

- `src/zig/array.zig` - Zig implementation with correct allocator pattern
- `src/main.rs` - Rust wrapper and tests
- `build.rs` - Build script with file cleanup

## Run

```bash
cargo run
cargo test
```

## Pattern Reference

```zig
// Zig: Local allocator + non-optional return
var gpa_instance = std.heap.GeneralPurposeAllocator(.{}){};
const allocator = gpa_instance.allocator();

export fn create() *MyStruct {
    const ptr = allocator.create(MyStruct) catch unreachable;
    ptr.* = MyStruct.init();
    return ptr;
}
```

```rust
// Rust: Raw pointer FFI
include_zig!("src/zig/file.zig", {
    fn create() -> *mut MyStruct;  // Not Option<*mut>
});
```
