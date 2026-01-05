use autozig::include_zig;

// 引用 wrapper.zig 中的函数
// wrapper.zig 内部会调用 C 代码（math.c）
include_zig!("src/wrapper.zig", {
    fn add(a: i32, b: i32) -> i32;
    fn multiply(a: i32, b: i32) -> i32;
    fn power(base: i32, exp: u32) -> i32;
    fn sum_array(arr: &[i32]) -> i32;
    fn string_length(s: &str) -> u32;
    fn average(arr: &[i32]) -> f64;
});

fn main() {
    println!("=== Zig-C 互操作示例 ===\n");
    println!("演示调用链：Rust → Zig → C\n");

    // 测试基础 C 函数（通过 Zig 包装）
    println!("1. 基础 C 函数调用:");
    println!("   add(10, 20) = {}", add(10, 20));
    println!("   multiply(7, 8) = {}", multiply(7, 8));

    // 测试 Zig 增强功能（使用 C 函数实现）
    println!("\n2. Zig 增强功能（使用 C multiply 实现幂运算）:");
    println!("   power(2, 10) = {}", power(2, 10));
    println!("   power(5, 3) = {}", power(5, 3));

    // 测试智能降级：&[i32] → ptr + len
    println!("\n3. 智能降级测试 - 数组切片:");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("   数组: {:?}", numbers);
    println!("   sum_array(&numbers) = {}", sum_array(&numbers));
    println!("   （Rust的 &[i32] 自动转换为 ptr + len 传递给 Zig）");

    // 测试智能降级：&str → ptr + len
    println!("\n4. 智能降级测试 - 字符串:");
    let test_str = "Hello from Rust!";
    println!("   字符串: \"{}\"", test_str);
    println!("   string_length(\"{}\") = {}", test_str, string_length(test_str));
    println!("   （Rust的 &str 自动转换为 ptr + len 传递给 Zig）");

    // 测试混合功能：C求和 + Zig浮点计算
    println!("\n5. 混合功能测试（C求和 + Zig浮点计算）:");
    println!("   average(&numbers) = {:.2}", average(&numbers));

    // 综合测试
    println!("\n6. 综合测试:");
    let data = vec![100, 200, 300, 400, 500];
    println!("   数据: {:?}", data);
    println!("   总和: {}", sum_array(&data));
    println!("   平均值: {:.2}", average(&data));

    // 计算所有元素的平方和
    let squares: Vec<i32> = data.iter().map(|&x| multiply(x, x)).collect();
    println!("   平方: {:?}", squares);
    println!("   平方和: {}", sum_array(&squares));

    println!("\n=== 演示完成 ===");
    println!("\n说明:");
    println!("- C 代码 (math.c) 提供基础运算");
    println!("- Zig 代码 (wrapper.zig) 包装C函数并添加新功能");
    println!("- Rust 代码 通过 autozig 宏调用 Zig 函数");
    println!("- 完整调用链: Rust → Zig → C");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        assert_eq!(add(5, 3), 8);
        assert_eq!(multiply(4, 7), 28);
    }

    #[test]
    fn test_power() {
        assert_eq!(power(2, 3), 8);
        assert_eq!(power(5, 2), 25);
        assert_eq!(power(10, 0), 1);
    }

    #[test]
    fn test_array_operations() {
        let arr = vec![1, 2, 3, 4, 5];
        assert_eq!(sum_array(&arr), 15);
        assert!((average(&arr) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_string_length() {
        assert_eq!(string_length("hello"), 5);
        assert_eq!(string_length(""), 0);
        assert_eq!(string_length("Rust + Zig + C"), 14);
    }
}
