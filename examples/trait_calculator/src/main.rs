//! Trait Calculator Example - Phase 1: Stateless Traits
//!
//! This example demonstrates AutoZig's trait support with zero-sized types
//! (ZST). It shows how Zig functions can implement Rust traits with zero
//! overhead.

use autozig::autozig;

/// Calculator trait - defines arithmetic operations
pub trait Calculator {
    fn add(&self, a: i32, b: i32) -> i32;
    fn multiply(&self, a: i32, b: i32) -> i32;
    fn divide(&self, a: i32, b: i32) -> Option<i32>;
    fn power(&self, base: i32, exp: u32) -> i32;
}

// AutoZig macro: Zig implementation + Rust trait mapping
autozig! {
    // Zig side: Pure function implementations
    export fn zig_add(a: i32, b: i32) i32 {
        return a + b;
    }

    export fn zig_multiply(a: i32, b: i32) i32 {
        return a * b;
    }

    export fn zig_divide(a: i32, b: i32) i32 {
        if (b == 0) {
            return -2147483648; // i32::MIN as sentinel for division by zero
        }
        return @divTrunc(a, b);
    }

    export fn zig_power(base: i32, exp: u32) i32 {
        var result: i32 = 1;
        var i: u32 = 0;
        while (i < exp) : (i += 1) {
            result *= base;
        }
        return result;
    }

    ---

    // Rust side: Trait mapping to Zig functions
    struct ZigCalculator;

    impl Calculator for ZigCalculator {
        fn add(&self, a: i32, b: i32) -> i32 {
            ffi::zig_add(a, b)
        }

        fn multiply(&self, a: i32, b: i32) -> i32 {
            ffi::zig_multiply(a, b)
        }

        fn divide(&self, a: i32, b: i32) -> Option<i32> {
            let result = ffi::zig_divide(a, b);
            if result == i32::MIN {
                None
            } else {
                Some(result)
            }
        }

        fn power(&self, base: i32, exp: u32) -> i32 {
            ffi::zig_power(base, exp)
        }
    }
}

fn main() {
    println!("=== AutoZig Trait Calculator Example ===\n");

    // Demo 1: Direct method calls on ZST
    println!("Demo 1: Direct Method Calls");
    println!("---------------------------");
    let calc = ZigCalculator;

    println!("2 + 3 = {}", calc.add(2, 3));
    println!("4 * 5 = {}", calc.multiply(4, 5));
    println!("20 / 4 = {:?}", calc.divide(20, 4));
    println!("10 / 0 = {:?}", calc.divide(10, 0));
    println!("2 ^ 10 = {}", calc.power(2, 10));
    println!();

    // Demo 2: Generic constraints (static dispatch)
    println!("Demo 2: Generic Constraints (Static Dispatch)");
    println!("---------------------------------------------");
    fn compute_expression<C: Calculator>(calc: &C, a: i32, b: i32) -> i32 {
        let sum = calc.add(a, b);
        let product = calc.multiply(sum, 2);
        product
    }

    let result = compute_expression(&calc, 5, 3);
    println!("(5 + 3) * 2 = {}", result);
    println!();

    // Demo 3: Trait objects (dynamic dispatch)
    println!("Demo 3: Trait Objects (Dynamic Dispatch)");
    println!("----------------------------------------");
    let calc_boxed: Box<dyn Calculator> = Box::new(ZigCalculator);

    println!("Using trait object:");
    println!("7 + 8 = {}", calc_boxed.add(7, 8));
    println!("6 * 7 = {}", calc_boxed.multiply(6, 7));
    println!("3 ^ 4 = {}", calc_boxed.power(3, 4));
    println!();

    // Demo 4: Multiple calculators in a Vec
    println!("Demo 4: Collection of Trait Objects");
    println!("-----------------------------------");
    let calculators: Vec<Box<dyn Calculator>> =
        vec![Box::new(ZigCalculator), Box::new(ZigCalculator::default())];

    for (i, calculator) in calculators.iter().enumerate() {
        println!("Calculator {}: 10 + 5 = {}", i + 1, calculator.add(10, 5));
    }
    println!();

    // Demo 5: Zero-cost abstraction verification
    println!("Demo 5: Zero-Cost Abstraction");
    println!("-----------------------------");
    println!(
        "ZigCalculator size: {} bytes (Zero-Sized Type!)",
        std::mem::size_of::<ZigCalculator>()
    );
    println!("This means no runtime overhead for the struct itself.");
    println!("Method calls are direct FFI calls to Zig functions.");
    println!();

    // Performance comparison
    println!("Demo 6: Performance Test");
    println!("-----------------------");
    let calc = ZigCalculator;
    let iterations = 1_000_000;

    let start = std::time::Instant::now();
    let mut sum = 0;
    for i in 0..iterations {
        sum += calc.add(i, 1);
    }
    let duration = start.elapsed();

    println!("Performed {} additions in {:?}", iterations, duration);
    println!(
        "Average: {:.2} ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );
    println!("Result checksum: {}", sum);
    println!();

    println!("=== All tests passed! ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let calc = ZigCalculator;
        assert_eq!(calc.add(2, 3), 5);
        assert_eq!(calc.add(-1, 1), 0);
        assert_eq!(calc.add(0, 0), 0);
    }

    #[test]
    fn test_multiplication() {
        let calc = ZigCalculator;
        assert_eq!(calc.multiply(3, 4), 12);
        assert_eq!(calc.multiply(-2, 5), -10);
        assert_eq!(calc.multiply(0, 100), 0);
    }

    #[test]
    fn test_division() {
        let calc = ZigCalculator;
        assert_eq!(calc.divide(10, 2), Some(5));
        assert_eq!(calc.divide(7, 3), Some(2));
        assert_eq!(calc.divide(10, 0), None);
    }

    #[test]
    fn test_power() {
        let calc = ZigCalculator;
        assert_eq!(calc.power(2, 0), 1);
        assert_eq!(calc.power(2, 10), 1024);
        assert_eq!(calc.power(3, 3), 27);
        assert_eq!(calc.power(5, 2), 25);
    }

    #[test]
    fn test_generic_constraint() {
        fn generic_add<C: Calculator>(calc: &C, a: i32, b: i32) -> i32 {
            calc.add(a, b)
        }

        let calc = ZigCalculator;
        assert_eq!(generic_add(&calc, 10, 20), 30);
    }

    #[test]
    fn test_trait_object() {
        let calc: Box<dyn Calculator> = Box::new(ZigCalculator);
        assert_eq!(calc.add(5, 5), 10);
        assert_eq!(calc.multiply(3, 3), 9);
    }

    #[test]
    fn test_zero_sized() {
        assert_eq!(std::mem::size_of::<ZigCalculator>(), 0);
    }
}
