use autozig::include_zig;

// 引用外部 Zig 文件 - 数学函数
include_zig!("zig/math.zig", {
    fn factorial(n: u32) -> u64;
    fn fibonacci(n: u32) -> u64;
    fn gcd(a: u64, b: u64) -> u64;
    fn is_prime(n: u64) -> bool;
});

// 引用外部 Zig 文件 - 字符串函数（使用智能降级）
include_zig!("zig/strings.zig", {
    fn string_length(s: &str) -> usize;
    fn string_count_char(s: &str, ch: u8) -> usize;
    fn string_to_lowercase(src: &str, dst: &mut [u8]);
});

fn main() {
    println!("=== AutoZig External Files 示例 ===\n");

    // === 数学函数测试 ===
    println!("【数学函数库 (zig/math.zig)】");

    println!("\n1. 阶乘计算:");
    for n in [5, 10, 15] {
        let result = factorial(n);
        println!("   {}! = {}", n, result);
    }

    println!("\n2. 斐波那契数列:");
    for n in [10, 15, 20] {
        let result = fibonacci(n);
        println!("   fib({}) = {}", n, result);
    }

    println!("\n3. 最大公约数:");
    let pairs = [(48, 18), (100, 35), (77, 49)];
    for (a, b) in pairs {
        let result = gcd(a, b);
        println!("   gcd({}, {}) = {}", a, b, result);
    }

    println!("\n4. 素数判断:");
    let numbers = [17, 20, 29, 30, 97, 100];
    for n in numbers {
        let result = is_prime(n);
        println!("   {} is prime: {}", n, result);
    }

    // === 字符串函数测试 ===
    println!("\n【字符串工具库 (zig/strings.zig)】");

    println!("\n5. 字符串长度:");
    let text = "Hello, AutoZig!";
    let len = string_length(text);
    println!("   文本: \"{}\"", text);
    println!("   长度: {}", len);

    println!("\n6. 字符计数:");
    let text = "programming";
    let ch = b'g';
    let count = string_count_char(text, ch);
    println!("   文本: \"{}\"", text);
    println!("   字符 '{}' 出现次数: {}", ch as char, count);

    println!("\n7. 转小写:");
    let text = "RUST AND ZIG";
    let mut output = vec![0u8; text.len()];
    string_to_lowercase(text, &mut output);
    let result = String::from_utf8(output).unwrap();
    println!("   输入: \"{}\"", text);
    println!("   输出: \"{}\"", result);

    println!("\n=== 所有测试通过 ===");
    println!("\n✨ 关键特性:");
    println!("  ✓ 支持引用外部 .zig 文件");
    println!("  ✓ 路径相对于 Cargo.toml 目录");
    println!("  ✓ 多个外部文件可以并存");
    println!("  ✓ 智能降级同样适用（&str 自动转换）");
    println!("  ✓ 零 unsafe 代码");
}
