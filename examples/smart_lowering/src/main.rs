use autozig::autozig;

autozig! {
    // Zig code - 字符串和切片处理
    const std = @import("std");
    
    // 字符串处理
    export fn process_string(ptr: [*]const u8, len: usize) usize {
        const s = ptr[0..len];
        var count: usize = 0;
        for (s) |c| {
            if (c == 'a' or c == 'e' or c == 'i' or c == 'o' or c == 'u') {
                count += 1;
            }
        }
        return count;
    }
    
    export fn to_uppercase(ptr: [*]const u8, len: usize, out_ptr: [*]u8, out_len: usize) void {
        const s = ptr[0..len];
        const out = out_ptr[0..out_len];
        for (s, 0..) |c, i| {
            if (i >= out.len) break;
            if (c >= 'a' and c <= 'z') {
                out[i] = c - 32;
            } else {
                out[i] = c;
            }
        }
    }
    
    // 切片处理
    export fn sum_array(ptr: [*]const i32, len: usize) i32 {
        const arr = ptr[0..len];
        var sum: i32 = 0;
        for (arr) |val| {
            sum += val;
        }
        return sum;
    }
    
    export fn max_array(ptr: [*]const i32, len: usize) i32 {
        const arr = ptr[0..len];
        if (arr.len == 0) return 0;
        var max_val = arr[0];
        for (arr[1..]) |val| {
            if (val > max_val) {
                max_val = val;
            }
        }
        return max_val;
    }
    
    export fn double_array(ptr: [*]i32, len: usize) void {
        const arr = ptr[0..len];
        for (arr) |*val| {
            val.* *= 2;
        }
    }
    
    export fn count_even(ptr: [*]const i32, len: usize) usize {
        const arr = ptr[0..len];
        var count: usize = 0;
        for (arr) |val| {
            if (@mod(val, 2) == 0) {
                count += 1;
            }
        }
        return count;
    }
    
    export fn checksum(ptr: [*]const u8, len: usize) u8 {
        const data = ptr[0..len];
        var sum: u8 = 0;
        for (data) |byte| {
            sum = sum +% byte;
        }
        return sum;
    }
    
    ---
    
    // Rust 签名 - 使用高级类型 &str, &[T], &mut [T]
    fn process_string(s: &str) -> usize;
    fn to_uppercase(input: &str, output: &mut [u8]);
    fn sum_array(arr: &[i32]) -> i32;
    fn max_array(arr: &[i32]) -> i32;
    fn double_array(arr: &mut [i32]);
    fn count_even(arr: &[i32]) -> usize;
    fn checksum(data: &[u8]) -> u8;
}

fn main() {
    println!("=== Smart Lowering 示例 ===\n");
    
    // 1. 字符串处理 - &str 自动降级为 ptr+len
    println!("1. 字符串处理");
    let text = "Hello, autozig!";
    let vowel_count = process_string(text);
    println!("  文本: \"{}\"", text);
    println!("  元音字母数: {}", vowel_count);
    
    // 2. 字符串转换
    println!("\n2. 字符串转大写");
    let input = "rust and zig";
    let mut output = vec![0u8; input.len()];
    to_uppercase(input, &mut output);
    let result = String::from_utf8(output).unwrap();
    println!("  输入: \"{}\"", input);
    println!("  输出: \"{}\"", result);
    
    // 3. 数组求和 - &[i32] 自动降级为 ptr+len
    println!("\n3. 数组求和");
    let numbers = vec![1, 2, 3, 4, 5];
    let sum = sum_array(&numbers);
    println!("  数组: {:?}", numbers);
    println!("  总和: {}", sum);
    
    // 4. 数组最大值
    println!("\n4. 数组最大值");
    let numbers = vec![42, 17, 99, 33, 8];
    let max = max_array(&numbers);
    println!("  数组: {:?}", numbers);
    println!("  最大值: {}", max);
    
    // 5. 数组修改 - &mut [i32] 自动降级为 mut_ptr+len
    println!("\n5. 数组修改（乘以2）");
    let mut numbers = vec![1, 2, 3, 4, 5];
    println!("  修改前: {:?}", numbers);
    double_array(&mut numbers);
    println!("  修改后: {:?}", numbers);
    
    // 6. 条件统计
    println!("\n6. 统计偶数");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let even_count = count_even(&numbers);
    println!("  数组: {:?}", numbers);
    println!("  偶数个数: {}", even_count);
    
    // 7. 字节数组处理 - &[u8] 自动降级为 ptr+len
    println!("\n7. 字节数组校验和");
    let data = b"Hello, World!";
    let sum = checksum(data);
    println!("  数据: {:?}", data);
    println!("  校验和: 0x{:02X}", sum);
    
    println!("\n=== 智能降级演示完成 ===");
    println!("\n关键特性:");
    println!("  ✓ &str 自动转换为 (*const u8, usize)");
    println!("  ✓ &[T] 自动转换为 (*const T, usize)");
    println!("  ✓ &mut [T] 自动转换为 (*mut T, usize)");
    println!("  ✓ 零 unsafe 代码");
    println!("  ✓ 类型安全");
    println!("  ✓ 符合 Rust 习惯用法");
}