//! Zero-Copy Buffer Passing Example (Phase 4.2)
//!
//! This example demonstrates zero-copy data transfer between Zig and Rust.
//! No serialization or copying occurs - Zig allocates memory and directly
//! transfers ownership to Rust's Vec.

use std::time::Instant;

use autozig::{
    include_zig,
    zero_copy::{
        RawVec,
        ZeroCopyBuffer,
    },
};

// 引用外部 Zig 文件 - 零拷贝实现
include_zig!("src/zero_copy.zig", {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct DataStats {
        pub min: i32,
        pub max: i32,
        pub sum: i64,
        pub count: usize,
    }

    fn generate_i32_data(size: usize) -> RawVec<i32>;
    fn generate_f32_data(size: usize) -> RawVec<f32>;
    fn generate_image_data(width: usize, height: usize) -> RawVec<u8>;
    fn compute_stats(data: *const i32, len: usize) -> DataStats;
    fn double_values(data: *mut i32, len: usize) -> ();
});

fn main() {
    println!("=== AutoZig Zero-Copy Buffer Example ===\n");

    // Example 1: Small data transfer
    demo_small_data();

    // Example 2: Large data transfer (>1MB)
    demo_large_data();

    // Example 3: Image data (simulating real-world use case)
    demo_image_data();

    // Example 4: In-place mutation
    demo_mutation();

    // Example 5: Performance comparison
    demo_performance();
}

fn demo_small_data() {
    println!("1. Small Data Transfer (1000 elements)");
    println!("   Transferring integer array from Zig to Rust...");

    let raw = generate_i32_data(1000);
    let buffer = ZeroCopyBuffer::from_zig_vec(raw);

    println!("   ✓ Received {} elements", buffer.len());
    println!("   ✓ First 10 elements: {:?}", &buffer.as_slice()[0..10]);
    println!("   ✓ Last 10 elements: {:?}", &buffer.as_slice()[990..1000]);

    // Convert to Vec (still zero-copy, just ownership transfer)
    let vec: Vec<i32> = buffer.into_vec();
    println!("   ✓ Converted to Vec<i32> with {} elements\n", vec.len());
}

fn demo_large_data() {
    println!("2. Large Data Transfer (>1MB)");
    let size = 1_000_000; // 1 million i32 values = 4MB
    println!("   Generating {} elements ({} MB)...", size, (size * 4) / 1_000_000);

    let start = Instant::now();
    let raw = generate_i32_data(size);
    let buffer = ZeroCopyBuffer::from_zig_vec(raw);
    let elapsed = start.elapsed();

    println!("   ✓ Transfer completed in {:?}", elapsed);
    println!("   ✓ Received {} elements", buffer.len());
    println!("   ✓ Memory address: {:p}", buffer.as_slice().as_ptr());

    // Verify data integrity
    let slice = buffer.as_slice();
    assert_eq!(slice[0], 0);
    assert_eq!(slice[100], 100);
    assert_eq!(slice[size - 1], (size - 1) as i32);
    println!("   ✓ Data integrity verified\n");

    // Compute statistics using Zig
    let stats = compute_stats(slice.as_ptr(), slice.len());
    println!("   Statistics computed by Zig:");
    println!(
        "     Min: {}, Max: {}, Sum: {}, Count: {}",
        stats.min, stats.max, stats.sum, stats.count
    );
    println!();
}

fn demo_image_data() {
    println!("3. Image Data Transfer (simulated)");
    let width = 1920;
    let height = 1080;
    let expected_size = width * height * 4; // RGBA
    println!(
        "   Generating {}x{} RGBA image ({} MB)...",
        width,
        height,
        expected_size / 1_000_000
    );

    let start = Instant::now();
    let raw = generate_image_data(width, height);
    let buffer = ZeroCopyBuffer::from_zig_vec(raw);
    let elapsed = start.elapsed();

    println!("   ✓ Transfer completed in {:?}", elapsed);
    println!("   ✓ Received {} bytes", buffer.len());
    assert_eq!(buffer.len(), expected_size);

    // Sample some pixels
    let pixels = buffer.as_slice();
    println!(
        "   ✓ First pixel (R,G,B,A): ({},{},{},{})",
        pixels[0], pixels[1], pixels[2], pixels[3]
    );
    println!(
        "   ✓ Last pixel (R,G,B,A): ({},{},{},{})",
        pixels[expected_size - 4],
        pixels[expected_size - 3],
        pixels[expected_size - 2],
        pixels[expected_size - 1]
    );
    println!();
}

fn demo_mutation() {
    println!("4. In-Place Mutation (Zero-Copy Both Ways)");
    let size = 10;
    println!("   Creating Rust Vec with {} elements...", size);

    let mut vec: Vec<i32> = (0..size as i32).collect();
    println!("   Original: {:?}", vec);

    // Pass to Zig for mutation
    double_values(vec.as_mut_ptr(), vec.len());
    println!("   After doubling (by Zig): {:?}", vec);
    println!("   ✓ Values doubled in-place, no copy\n");
}

fn demo_performance() {
    println!("5. Performance Comparison");
    let size = 10_000_000; // 10 million elements

    // Zero-copy transfer
    print!("   Zero-copy method: ");
    let start = Instant::now();
    let raw = generate_i32_data(size);
    let buffer = ZeroCopyBuffer::from_zig_vec(raw);
    let vec_zero_copy: Vec<i32> = buffer.into_vec();
    let zero_copy_time = start.elapsed();
    println!("{:?} for {} elements", zero_copy_time, vec_zero_copy.len());

    // Simulate copy-based transfer (for comparison)
    print!("   Copy-based method (simulated): ");
    let start = Instant::now();
    let raw2 = generate_i32_data(size);
    // Create a temporary buffer to get a slice
    let temp_buffer = ZeroCopyBuffer::from_zig_vec(raw2);
    let vec_copied: Vec<i32> = temp_buffer.as_slice().to_vec(); // This copies
                                                                // temp_buffer will be dropped here, cleaning up the allocation
    let copy_time = start.elapsed();
    println!("{:?} for {} elements", copy_time, vec_copied.len());

    let speedup = copy_time.as_nanos() as f64 / zero_copy_time.as_nanos() as f64;
    println!("\n   ✓ Zero-copy is {:.2}x faster!", speedup);
    println!("   ✓ Saved {} ms by avoiding copy", (copy_time - zero_copy_time).as_millis());
}
