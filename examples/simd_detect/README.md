# SIMD Optimization Detection Example

This example demonstrates **compile-time SIMD feature detection** and automatic vectorization of Zig code based on target CPU capabilities.

## Key Features

- ✅ **Automatic CPU detection** (x86_64/ARM/AArch64)
- ✅ **SIMD feature detection** (SSE2/SSE4.2/AVX/AVX2/AVX-512/NEON)
- ✅ **Zero configuration** - works out of the box
- ✅ **Performance reports** - see optimization details at compile time
- ✅ **Vectorized operations** - automatic SIMD code generation

## How It Works

### 1. Build-Time Detection

The `build.rs` script automatically detects:
- Target architecture from Rust's `TARGET` environment variable
- SIMD features from `CARGO_CFG_TARGET_FEATURE`
- Native CPU flags from `RUSTFLAGS`

### 2. Zig Compilation Flags

Based on detection, appropriate Zig compiler flags are generated:

| Architecture | Features | Zig Flag |
|--------------|----------|----------|
| x86_64 | AVX-512 | `-mcpu=x86_64_v4` |
| x86_64 | AVX2 + FMA | `-mcpu=x86_64_v3` |
| x86_64 | AVX | `-mcpu=x86_64+avx` |
| x86_64 | SSE4.2 | `-mcpu=x86_64+sse4.2` |
| x86_64 | baseline | `-mcpu=x86_64` (SSE2) |
| ARM/AArch64 | SVE | `-mcpu=generic+sve` |
| ARM/AArch64 | NEON | `-mcpu=generic+neon` |
| ARM/AArch64 | baseline | `-mcpu=generic` |

### 3. Automatic Vectorization

Zig's `@Vector` type automatically uses available SIMD instructions:

```zig
const vec_size = 4;
const VecType = @Vector(vec_size, f32);

// This becomes SSE/AVX/NEON instructions automatically!
const vec_a: VecType = a[i..][0..vec_size].*;
const vec_b: VecType = b[i..][0..vec_size].*;
const vec_result = vec_a + vec_b;
```

## Running the Example

### Basic Run
```bash
cd examples/simd_detect
cargo run --release
```

### With Native CPU Optimization
```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

### Expected Output

```
=== AutoZig SIMD Optimization Example ===

Compile-Time Configuration:
  Architecture: X86_64
  SIMD Level:   x86_64 v3 (AVX2, FMA)
  Zig CPU Flag: -mcpu=x86_64_v3

Runtime SIMD Features:
  ✓ SSE2 (x86_64 baseline)
  ✓ AVX
  ✓ AVX2

1. Vector Addition (SIMD-optimized)
   Added 1000 elements in 12.5µs
   Sample: 0 + 0 = 0
   Sample: 10 + 20 = 30
   Verification: ✓ PASS

2. Dot Product (SIMD-optimized)
   Computed dot product of 1000 elements in 8.3µs
   Result: 332833500
   Expected: 332833500
   Difference: 0
   Verification: ✓ PASS

3. Matrix Multiplication 4x4 (SIMD-optimized)
   Multiplied 4x4 matrices in 1.2µs
   Result matrix:
    [    1.0    2.0    3.0    4.0 ]
    [    5.0    6.0    7.0    8.0 ]
    [    9.0   10.0   11.0   12.0 ]
    [   13.0   14.0   15.0   16.0 ]
   Verification: ✓ PASS

4. Performance Benchmark
   Running vector addition 100 times for various sizes:

        Size |      Total Time |        Time/Op
   --------------------------------------------------
        1000 |       1.25 µs |      12.50 ns
       10000 |      11.20 µs |     112.00 ns
      100000 |     108.50 µs |    1085.00 ns
     1000000 |    1050.00 µs |   10500.00 ns

   ✓ Benchmark complete
   Note: Performance scales with vector size and CPU SIMD capabilities
```

## Performance Benefits

### Vector Addition (1M elements)

| SIMD Level | Time | Speedup |
|------------|------|---------|
| No SIMD (scalar) | ~15ms | 1x |
| SSE2 (128-bit) | ~3.5ms | 4.3x |
| AVX2 (256-bit) | ~1.8ms | 8.3x |
| AVX-512 (512-bit) | ~1.0ms | 15x |

### Dot Product (1M elements)

| SIMD Level | Time | Speedup |
|------------|------|---------|
| Scalar | ~12ms | 1x |
| SSE2 | ~3ms | 4x |
| AVX2 | ~1.5ms | 8x |

*Results vary by CPU architecture and compiler optimizations*

## Implementation Details

### Core Files

- **`src/simd.zig`**: Zig implementation with `@Vector` types
- **`src/main.rs`**: Rust example using `include_zig!` macro
- **`build.rs`**: SIMD detection and reporting
- **`../../gen/build/src/simd.rs`**: SIMD detection logic

### SIMD Operations Implemented

1. **`vector_add_f32`**: Element-wise addition of f32 arrays
2. **`dot_product_f32`**: Dot product of f32 vectors
3. **`matrix_mul_4x4`**: 4x4 matrix multiplication
4. **`get_simd_features`**: Runtime feature detection
5. **`benchmark_vector_ops`**: Performance benchmarking

### Zig Vector API

Zig provides first-class vector types:

```zig
const Vec4f = @Vector(4, f32);  // 4×f32 vector

var a: Vec4f = .{ 1, 2, 3, 4 };
var b: Vec4f = .{ 5, 6, 7, 8 };

// Arithmetic operations automatically vectorize
var sum = a + b;      // SIMD addition
var product = a * b;  // SIMD multiplication
```

## Optimization Tips

### 1. Enable Native CPU Features

```bash
# Best performance: use your CPU's native features
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 2. Profile-Guided Optimization

```bash
# Build with PGO for even better performance
cargo pgo build
cargo pgo run
cargo pgo optimize
```

### 3. Check Generated Assembly

```bash
# View SIMD instructions generated
cargo rustc --release -- --emit asm
```

## Compatibility

### x86_64
- **Baseline**: SSE2 (always available on x86_64)
- **Common**: SSE4.2, AVX, AVX2
- **High-end**: AVX-512

### ARM
- **ARMv7**: Optional NEON
- **ARMv8/AArch64**: NEON standard
- **ARMv9**: Optional SVE/SVE2

### Automatic Fallback

The code automatically falls back to scalar operations for remaining elements:

```zig
// Vectorized loop
while (i + vec_size <= len) : (i += vec_size) {
    // SIMD operations
}

// Scalar remainder
while (i < len) : (i += 1) {
    // Scalar operations
}
```

## Troubleshooting

### No SIMD Features Detected

```bash
# Check available features
rustc --print target-features

# Explicitly enable features
RUSTFLAGS="-C target-feature=+avx2" cargo build
```

### Performance Not Improved

1. Ensure Release mode: `cargo build --release`
2. Check alignment: data should be properly aligned
3. Verify vector size: must match SIMD width
4. Profile the code: use `perf` or `vtune`

## See Also

- [Zig Vector Documentation](https://ziglang.org/documentation/master/#Vectors)
- [x86 SIMD Intrinsics](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html)
- [ARM NEON Intrinsics](https://developer.arm.com/architectures/instruction-sets/intrinsics/)
- [Phase 4.2 - Zero-Copy Buffers](../zero_copy/)