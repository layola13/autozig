use autozig::include_zig;

// 引用 zig.zig 文件中的函数
include_zig!("zig/zig.zig", {
    fn add(a: i32, b: i32) -> i32;
    fn printing(s: &str);
    fn itoa_u64(n: u64, buff: &mut [u8]);
});

fn main() {
    println!("=== 测试 zig.zig 库函数 ===\n");
    
    // 测试 add 函数
    println!("1. 测试 add 函数:");
    let result = add(10, 20);
    println!("   add(10, 20) = {}", result);
    
    // 测试 printing 函数
    println!("\n2. 测试 printing 函数:");
    println!("   调用 printing(\"Hello from Rust!\"):");
    printing("Hello from Rust!");
    
    // 测试 itoa_u64 函数
    println!("\n3. 测试 itoa_u64 函数:");
    let numbers = [123u64, 456789, 1234567890];
    for num in numbers {
        let mut buffer = vec![0u8; 20];
        itoa_u64(num, &mut buffer);
        
        // 找到第一个非零字节开始的位置
        let start = buffer.iter().position(|&x| x != 0).unwrap_or(0);
        let result_str = String::from_utf8_lossy(&buffer[start..]);
        let result_str = result_str.trim_end_matches('\0');
        
        println!("   itoa_u64({}) = \"{}\"", num, result_str);
    }
    
    println!("\n=== 所有测试完成 ===");
}