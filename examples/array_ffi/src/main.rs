//! Array FFI Example
//! 
//! This example demonstrates the new array support in autozig:
//! 1. Fixed-size arrays as parameters [T; N]
//! 2. Mutable array references &mut [T; N]
//! 3. Arrays as return values -> [T; N]

use autozig::autozig;

autozig! {
    // Zig code section
    const std = @import("std");

    // 1. Array as parameter (read-only)
    export fn array_sum(arr: *const [5]i32) i32 {
        var sum: i32 = 0;
        for (arr) |val| {
            sum += val;
        }
        return sum;
    }

    // 2. Mutable array parameter (in-place modification)
    export fn array_double(arr: *[5]i32) void {
        for (arr) |*val| {
            val.* *= 2;
        }
    }

    // 3. Array as return value
    export fn create_range() *const [5]i32 {
        const static_array = [_]i32{ 1, 2, 3, 4, 5 };
        return &static_array;
    }

    // 4. Matrix operation (2D array)
    export fn matrix_transpose(input: *const [3][3]i32, output: *[3][3]i32) void {
        var i: usize = 0;
        while (i < 3) : (i += 1) {
            var j: usize = 0;
            while (j < 3) : (j += 1) {
                output[j][i] = input[i][j];
            }
        }
    }

    ---

    // Rust function signatures
    fn array_sum(arr: [i32; 5]) -> i32;
    fn array_double(arr: &mut [i32; 5]);
    fn create_range() -> [i32; 5];
    fn matrix_transpose(input: [i32; 9], output: &mut [i32; 9]);
}

fn main() {
    println!("=== AutoZig Array FFI Examples ===\n");

    // Example 1: Read-only array parameter
    println!("1. Array Sum (read-only parameter)");
    let numbers = [10, 20, 30, 40, 50];
    let sum = array_sum(numbers);
    println!("   Input: {:?}", numbers);
    println!("   Sum: {}\n", sum);

    // Example 2: Mutable array parameter
    println!("2. Array Double (mutable parameter)");
    let mut values = [1, 2, 3, 4, 5];
    println!("   Before: {:?}", values);
    array_double(&mut values);
    println!("   After:  {:?}\n", values);

    // Example 3: Array return value
    println!("3. Create Range (array return)");
    let range = create_range();
    println!("   Result: {:?}\n", range);

    // Example 4: Matrix transpose
    println!("4. Matrix Transpose (2D array)");
    let matrix = [
        1, 2, 3,
        4, 5, 6,
        7, 8, 9,
    ];
    let mut transposed = [0; 9];
    println!("   Input matrix:");
    print_matrix(&matrix);
    matrix_transpose(matrix, &mut transposed);
    println!("   Transposed:");
    print_matrix(&transposed);

    println!("\n✅ All array FFI operations completed successfully!");
}

fn print_matrix(matrix: &[i32; 9]) {
    for i in 0..3 {
        print!("   ");
        for j in 0..3 {
            print!("{:3} ", matrix[i * 3 + j]);
        }
        println!();
    }
}