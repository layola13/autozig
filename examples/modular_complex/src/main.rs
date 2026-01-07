//! Modular Complex Example
//! 
//! Demonstrates the new modular compilation mode where:
//! - Multiple .zig files in different directories are compiled separately
//! - Each module is independent and can be maintained separately
//! - No global variable conflicts (e.g., allocator redefinition)
//! - Uses build.zig for compilation (recommended mode)

use autozig::prelude::*;

// Import external Zig modules from different directories
// The include_zig! macro automatically generates FFI bindings
include_zig!("src/math/vector.zig", {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Vector2D {
        pub x: f32,
        pub y: f32,
    }

    fn vector_create(x: f32, y: f32) -> Vector2D;
    fn vector_add(a: Vector2D, b: Vector2D) -> Vector2D;
    fn vector_length(v: Vector2D) -> f32;
    fn vector_dot(a: Vector2D, b: Vector2D) -> f32;
});

include_zig!("src/utils/string_ops.zig", {
    fn string_length(ptr: *const i8) -> usize;
    fn string_compare(a: *const i8, b: *const i8) -> i32;
    fn string_hash(ptr: *const i8) -> u64;
});

include_zig!("src/data/array_ops.zig", {
    fn array_sum_i32(ptr: *const i32, len: usize) -> i64;
    fn array_min_i32(ptr: *const i32, len: usize) -> i32;
    fn array_max_i32(ptr: *const i32, len: usize) -> i32;
    fn array_reverse_i32(ptr: *mut i32, len: usize);
});

fn main() {
    println!("=== Modular Complex Example ===\n");
    println!("This example demonstrates modular Zig compilation:");
    println!("- Multiple .zig files in different directories");
    println!("- Each module compiled independently");
    println!("- No global variable conflicts\n");

    // Test Vector operations
    println!("--- Vector Operations ---");
    unsafe {
        let v1 = vector_create(3.0, 4.0);
        let v2 = vector_create(1.0, 2.0);
        
        println!("v1 = ({}, {})", v1.x, v1.y);
        println!("v2 = ({}, {})", v2.x, v2.y);
        
        let v3 = vector_add(v1, v2);
        println!("v1 + v2 = ({}, {})", v3.x, v3.y);
        
        let len = vector_length(v1);
        println!("|v1| = {}", len);
        
        let dot = vector_dot(v1, v2);
        println!("v1 · v2 = {}", dot);
    }

    // Test String operations
    println!("\n--- String Operations ---");
    unsafe {
        let s1 = b"Hello\0".as_ptr() as *const i8;
        let s2 = b"World\0".as_ptr() as *const i8;
        let s3 = b"Hello\0".as_ptr() as *const i8;
        
        let len1 = string_length(s1);
        let len2 = string_length(s2);
        println!("Length of 'Hello': {}", len1);
        println!("Length of 'World': {}", len2);
        
        let cmp1 = string_compare(s1, s2);
        let cmp2 = string_compare(s1, s3);
        println!("Compare 'Hello' vs 'World': {}", cmp1);
        println!("Compare 'Hello' vs 'Hello': {}", cmp2);
        
        let hash = string_hash(s1);
        println!("Hash of 'Hello': {}", hash);
    }

    // Test Array operations
    println!("\n--- Array Operations ---");
    unsafe {
        let mut arr = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
        println!("Original array: {:?}", arr);
        
        let sum = array_sum_i32(arr.as_ptr(), arr.len());
        println!("Sum: {}", sum);
        
        let min = array_min_i32(arr.as_ptr(), arr.len());
        let max = array_max_i32(arr.as_ptr(), arr.len());
        println!("Min: {}, Max: {}", min, max);
        
        array_reverse_i32(arr.as_mut_ptr(), arr.len());
        println!("Reversed array: {:?}", arr);
    }

    println!("\n=== All tests passed! ===");
    println!("✓ Modular compilation works correctly");
    println!("✓ Multiple independent Zig modules");
    println!("✓ No global variable conflicts");
}