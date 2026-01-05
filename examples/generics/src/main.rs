//! Phase 3: Generic Functions Example
//! 
//! This example demonstrates AutoZig's generic monomorphization support.
//! Generic Rust functions are automatically monomorphized to specific types.

use autozig::autozig;

autozig! {
    // Zig implementations for different types
    const std = @import("std");
    
    // Sum function for i32
    export fn sum_i32(data_ptr: [*]const i32, data_len: usize) i32 {
        var total: i32 = 0;
        var i: usize = 0;
        while (i < data_len) : (i += 1) {
            total += data_ptr[i];
        }
        return total;
    }
    
    // Sum function for f64
    export fn sum_f64(data_ptr: [*]const f64, data_len: usize) f64 {
        var total: f64 = 0.0;
        var i: usize = 0;
        while (i < data_len) : (i += 1) {
            total += data_ptr[i];
        }
        return total;
    }
    
    // Sum function for u64
    export fn sum_u64(data_ptr: [*]const u64, data_len: usize) u64 {
        var total: u64 = 0;
        var i: usize = 0;
        while (i < data_len) : (i += 1) {
            total += data_ptr[i];
        }
        return total;
    }
    
    // Max function for i32
    export fn max_i32(data_ptr: [*]const i32, data_len: usize) i32 {
        if (data_len == 0) return 0;
        var maximum: i32 = data_ptr[0];
        var i: usize = 1;
        while (i < data_len) : (i += 1) {
            if (data_ptr[i] > maximum) {
                maximum = data_ptr[i];
            }
        }
        return maximum;
    }
    
    // Max function for f64
    export fn max_f64(data_ptr: [*]const f64, data_len: usize) f64 {
        if (data_len == 0) return 0.0;
        var maximum: f64 = data_ptr[0];
        var i: usize = 1;
        while (i < data_len) : (i += 1) {
            if (data_ptr[i] > maximum) {
                maximum = data_ptr[i];
            }
        }
        return maximum;
    }
    
    ---
    
    // Generic Rust signatures with monomorphization
    // The #[monomorphize(...)] attribute specifies which concrete types to generate
    
    /// Generic sum function - automatically generates sum_i32, sum_f64, sum_u64
    #[monomorphize(i32, f64, u64)]
    fn sum<T>(data: &[T]) -> T;
    
    /// Generic max function - automatically generates max_i32, max_f64
    #[monomorphize(i32, f64)]
    fn max<T>(data: &[T]) -> T;
}

fn main() {
    println!("=== AutoZig Phase 3: Generic Functions Demo ===\n");
    
    // Test sum with i32
    let integers = vec![1, 2, 3, 4, 5];
    let int_sum = sum_i32(&integers);
    println!("Sum of {:?} = {}", integers, int_sum);
    assert_eq!(int_sum, 15);
    
    // Test sum with f64
    let floats = vec![1.5, 2.5, 3.5, 4.5];
    let float_sum = sum_f64(&floats);
    println!("Sum of {:?} = {}", floats, float_sum);
    assert_eq!(float_sum, 12.0);
    
    // Test sum with u64
    let unsigneds = vec![100u64, 200, 300, 400];
    let unsigned_sum = sum_u64(&unsigneds);
    println!("Sum of {:?} = {}", unsigneds, unsigned_sum);
    assert_eq!(unsigned_sum, 1000);
    
    println!();
    
    // Test max with i32
    let int_data = vec![5, 2, 8, 1, 9, 3];
    let int_max = max_i32(&int_data);
    println!("Max of {:?} = {}", int_data, int_max);
    assert_eq!(int_max, 9);
    
    // Test max with f64
    let float_data = vec![3.14, 2.71, 1.41, 9.99, 0.57];
    let float_max = max_f64(&float_data);
    println!("Max of {:?} = {}", float_data, float_max);
    assert_eq!(float_max, 9.99);
    
    println!("\n✅ All generic function tests passed!");
    println!("\nPhase 3 Feature Summary:");
    println!("- ✅ Generic functions with monomorphization");
    println!("- ✅ Multiple concrete type instantiations (i32, f64, u64)");
    println!("- ✅ Automatic name mangling (sum<T> → sum_i32, sum_f64, etc.)");
    println!("- ✅ Type-safe FFI bindings for each monomorphized version");
}