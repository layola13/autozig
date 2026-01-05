const std = @import("std");

// 数学运算函数库

export fn factorial(n: u32) u64 {
    if (n <= 1) return 1;
    var result: u64 = 1;
    var i: u32 = 2;
    while (i <= n) : (i += 1) {
        result *= i;
    }
    return result;
}

export fn fibonacci(n: u32) u64 {
    if (n <= 1) return n;
    var a: u64 = 0;
    var b: u64 = 1;
    var i: u32 = 2;
    while (i <= n) : (i += 1) {
        const temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

export fn gcd(a: u64, b: u64) u64 {
    var x = a;
    var y = b;
    while (y != 0) {
        const temp = x % y;
        x = y;
        y = temp;
    }
    return x;
}

export fn is_prime(n: u64) bool {
    if (n < 2) return false;
    if (n == 2) return true;
    if (n % 2 == 0) return false;

    var i: u64 = 3;
    while (i * i <= n) : (i += 2) {
        if (n % i == 0) return false;
    }
    return true;
}

// Unit tests for math functions
test "factorial basic cases" {
    try std.testing.expectEqual(@as(u64, 1), factorial(0));
    try std.testing.expectEqual(@as(u64, 1), factorial(1));
    try std.testing.expectEqual(@as(u64, 2), factorial(2));
    try std.testing.expectEqual(@as(u64, 6), factorial(3));
    try std.testing.expectEqual(@as(u64, 24), factorial(4));
    try std.testing.expectEqual(@as(u64, 120), factorial(5));
}

test "fibonacci sequence" {
    try std.testing.expectEqual(@as(u64, 0), fibonacci(0));
    try std.testing.expectEqual(@as(u64, 1), fibonacci(1));
    try std.testing.expectEqual(@as(u64, 1), fibonacci(2));
    try std.testing.expectEqual(@as(u64, 2), fibonacci(3));
    try std.testing.expectEqual(@as(u64, 3), fibonacci(4));
    try std.testing.expectEqual(@as(u64, 5), fibonacci(5));
    try std.testing.expectEqual(@as(u64, 8), fibonacci(6));
}

test "gcd calculations" {
    try std.testing.expectEqual(@as(u64, 6), gcd(12, 18));
    try std.testing.expectEqual(@as(u64, 1), gcd(17, 19));
    try std.testing.expectEqual(@as(u64, 5), gcd(25, 15));
    try std.testing.expectEqual(@as(u64, 12), gcd(48, 60));
}

test "prime number check" {
    try std.testing.expect(!is_prime(0));
    try std.testing.expect(!is_prime(1));
    try std.testing.expect(is_prime(2));
    try std.testing.expect(is_prime(3));
    try std.testing.expect(!is_prime(4));
    try std.testing.expect(is_prime(5));
    try std.testing.expect(!is_prime(9));
    try std.testing.expect(is_prime(11));
    try std.testing.expect(is_prime(13));
    try std.testing.expect(!is_prime(15));
}
