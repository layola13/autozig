# Array FFI Example

This example demonstrates the new array support in autozig, including:

## Features

### 1. Fixed-Size Arrays as Parameters
```rust
fn array_sum(arr: [i32; 5]) -> i32;
```
- Arrays are automatically converted to pointers in FFI
- Read-only access to array data
- Zero-copy performance

### 2. Mutable Array References
```rust
fn array_double(arr: &mut [i32; 5]);
```
- In-place array modification
- Automatic conversion to mutable pointer
- Safe Rust wrapper ensures memory safety

### 3. Arrays as Return Values
```rust
fn create_range() -> [i32; 5];
```
- Return fixed-size arrays from Zig functions
- Automatic pointer dereferencing and copying
- Stack-allocated results

### 4. Multi-Dimensional Arrays
```rust
fn matrix_transpose(input: [i32; 9], output: &mut [i32; 9]);
```
- Support for 2D and higher-dimensional arrays
- Efficient matrix operations

## Building and Running

```bash
cd autozig/examples/array_ffi
cargo build
cargo run
```

## Expected Output

```
=== AutoZig Array FFI Examples ===

1. Array Sum (read-only parameter)
   Input: [10, 20, 30, 40, 50]
   Sum: 150

2. Array Double (mutable parameter)
   Before: [1, 2, 3, 4, 5]
   After:  [2, 4, 6, 8, 10]

3. Create Range (array return)
   Result: [1, 2, 3, 4, 5]

4. Matrix Transpose (2D array)
   Input matrix:
     1   2   3
     4   5   6
     7   8   9
   Transposed:
     1   4   7
     2   5   8
     3   6   9

✅ All array FFI operations completed successfully!
```

## Implementation Details

### FFI Lowering

autozig automatically handles array type conversions:

| Rust Type | FFI Type | Zig Type |
|-----------|----------|----------|
| `[T; N]` | `*const [T; N]` | `*const [N]T` |
| `&mut [T; N]` | `*mut T` | `*[N]T` |
| `-> [T; N]` | `-> *const [T; N]` | `*const [N]T` |

### Memory Safety

- Read-only arrays: Passed by const pointer, no copying
- Mutable arrays: Passed by mutable pointer, modifications visible to caller
- Return values: Copied from Zig static data to Rust stack

## Comparison with Slices

| Feature | Fixed Arrays `[T; N]` | Dynamic Slices `&[T]` |
|---------|----------------------|----------------------|
| Size | Compile-time known | Runtime determined |
| FFI Overhead | Single pointer | Pointer + length |
| Stack/Heap | Stack-allocated | Can be either |
| Use Case | Small, fixed data | Variable-length data |

## Advanced Usage

### Custom Array Sizes

```rust
autozig! {
    export fn process_large_array(data: *const [1024]f32) f32 {
        // Process 1024 floats
    }
    
    ---
    
    fn process_large_array(data: [f32; 1024]) -> f32;
}
```

### Nested Arrays

```rust
autozig! {
    export fn process_3d_tensor(tensor: *const [4][4][4]f32) void {
        // 3D array processing
    }
    
    ---
    
    fn process_3d_tensor(tensor: [f32; 64]);
}
```

## Performance Notes

- No runtime overhead for array parameter passing
- Return values require one memcpy operation
- Mutable parameters enable zero-copy in-place operations
- Suitable for SIMD-optimized Zig code

## Limitations

- Array size must be known at compile time
- For dynamic sizes, use slices (`&[T]`) instead
- Large arrays (>1KB) may cause stack overflow - consider heap allocation

## See Also

- [Slice example](../slice_example) - Dynamic-length data
- [Zero-copy example](../zero_copy) - Performance optimization
- [SIMD example](../simd) - Vectorized array operations