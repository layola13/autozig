//! SIMD Optimization Detection Example
//!
//! This example demonstrates compile-time SIMD feature detection and
//! automatic vectorization of Zig code based on target CPU.

use std::time::Instant;

use autozig::include_zig;

// 引用外部 Zig 文件 - SIMD 优化实现
include_zig!("src/simd.zig", {
    fn vector_add_f32(a: *const f32, b: *const f32, result: *mut f32, len: usize) -> ();
    fn dot_product_f32(a: *const f32, b: *const f32, len: usize) -> f32;
    fn matrix_mul_4x4(a: *const f32, b: *const f32, result: *mut f32) -> ();
    fn get_simd_features() -> u32;
    fn benchmark_vector_ops(size: usize, iterations: u32) -> u64;
});

fn main() {
    println!("=== AutoZig SIMD Optimization Example ===\n");

    // Display compile-time detected features
    display_compile_info();

    // Display runtime detected features
    display_runtime_features();

    // Demo 1: Vector addition
    demo_vector_add();

    // Demo 2: Dot product
    demo_dot_product();

    // Demo 3: Matrix multiplication
    demo_matrix_mul();

    // Demo 4: Performance benchmark
    demo_benchmark();
}

fn display_compile_info() {
    println!("Compile-Time Configuration:");
    println!("  Architecture: {}", env!("AUTOZIG_SIMD_ARCH"));
    println!("  SIMD Level:   {}", env!("AUTOZIG_SIMD_CONFIG"));
    println!("  Zig CPU Flag: {}\n", env!("AUTOZIG_ZIG_CPU_FLAG"));
}

fn display_runtime_features() {
    let features = get_simd_features();

    println!("Runtime SIMD Features:");

    if features & 0x01 != 0 {
        println!("  ✓ SSE2 (x86_64 baseline)");
    }
    if features & 0x02 != 0 {
        println!("  ✓ SSE4.2");
    }
    if features & 0x04 != 0 {
        println!("  ✓ AVX");
    }
    if features & 0x08 != 0 {
        println!("  ✓ AVX2");
    }
    if features & 0x10 != 0 {
        println!("  ✓ AVX-512");
    }
    if features & 0x100 != 0 {
        println!("  ✓ NEON (ARM)");
    }

    if features == 0 {
        println!("  (No SIMD features detected)");
    }

    println!();
}

fn demo_vector_add() {
    println!("1. Vector Addition (SIMD-optimized)");

    let size = 1000;
    let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
    let mut result = vec![0.0f32; size];

    let start = Instant::now();
    vector_add_f32(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), size);
    let elapsed = start.elapsed();

    println!("  Added {} elements in {:?}", size, elapsed);
    println!("  Sample: {} + {} = {}", a[0], b[0], result[0]);
    println!("  Sample: {} + {} = {}", a[10], b[10], result[10]);

    // Verify correctness
    let correct = result
        .iter()
        .enumerate()
        .all(|(i, &val)| val == (i as f32 + (i * 2) as f32));
    println!("  Verification: {}\n", if correct { "✓ PASS" } else { "✗ FAIL" });
}

fn demo_dot_product() {
    println!("2. Dot Product (SIMD-optimized)");

    let size = 1000;
    let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..size).map(|i| (i + 1) as f32).collect();

    let start = Instant::now();
    let result = dot_product_f32(a.as_ptr(), b.as_ptr(), size);
    let elapsed = start.elapsed();

    // Expected: sum of i * (i + 1) for i in 0..1000
    let expected: f32 = (0..size).map(|i| (i * (i + 1)) as f32).sum();

    println!("  Computed dot product of {} elements in {:?}", size, elapsed);
    println!("  Result: {}", result);
    println!("  Expected: {}", expected);
    println!("  Difference: {}", (result - expected).abs());
    println!(
        "  Verification: {}\n",
        if (result - expected).abs() < 0.01 {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
}

fn demo_matrix_mul() {
    println!("3. Matrix Multiplication 4x4 (SIMD-optimized)");

    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0,
    ];

    #[rustfmt::skip]
    let b = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];

    let mut result = [0.0f32; 16];

    let start = Instant::now();
    matrix_mul_4x4(a.as_ptr(), b.as_ptr(), result.as_mut_ptr());
    let elapsed = start.elapsed();

    println!("  Multiplied 4x4 matrices in {:?}", elapsed);
    println!("  Result matrix:");
    for row in 0..4 {
        print!("    [");
        for col in 0..4 {
            print!(" {:6.1}", result[row * 4 + col]);
        }
        println!(" ]");
    }

    // Verify (multiplying by identity should give original matrix)
    let correct = a
        .iter()
        .zip(result.iter())
        .all(|(a, r)| (a - r).abs() < 0.01);
    println!("  Verification: {}\n", if correct { "✓ PASS" } else { "✗ FAIL" });
}

fn demo_benchmark() {
    println!("4. Performance Benchmark");

    let sizes = [1000, 10_000, 100_000, 1_000_000];
    let iterations = 100;

    println!("  Running vector addition {} times for various sizes:\n", iterations);
    println!("  {:>10} | {:>15} | {:>15}", "Size", "Total Time", "Time/Op");
    println!("  {}", "-".repeat(50));

    for &size in &sizes {
        let total_ns = benchmark_vector_ops(size, iterations);

        let total_us = total_ns as f64 / 1000.0;
        let per_op_ns = total_ns as f64 / iterations as f64;

        println!("  {:>10} | {:>12.2} µs | {:>12.2} ns", size, total_us, per_op_ns);
    }

    println!("\n  ✓ Benchmark complete");
    println!("  Note: Performance scales with vector size and CPU SIMD capabilities");
}
