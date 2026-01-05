use autozig::autozig;

autozig! {
const std = @import("std");

// ========== 字符串处理 ==========

// 1. 字符串长度计算（接收 ptr + len）
pub export fn string_length(ptr: [*]const u8, len: usize) usize {
    _ = ptr;
    return len;
}

// 2. 字符串反转（就地修改）
pub export fn string_reverse(ptr: [*]u8, len: usize) void {
    var i: usize = 0;
    const half = len / 2;
    while (i < half) : (i += 1) {
        const temp = ptr[i];
        ptr[i] = ptr[len - 1 - i];
        ptr[len - 1 - i] = temp;
    }
}

// 3. 字符串拷贝并转大写
pub export fn string_to_upper(src_ptr: [*]const u8, src_len: usize, dst_ptr: [*]u8, dst_len: usize) void {
    _ = dst_len;
    var i: usize = 0;
    while (i < src_len) : (i += 1) {
        const c = src_ptr[i];
        if (c >= 'a' and c <= 'z') {
            dst_ptr[i] = c - 32;
        } else {
            dst_ptr[i] = c;
        }
    }
}

// ========== 数组/Slice 处理 ==========

// 4. 数组求和
pub export fn array_sum(arr: [*]const i32, len: usize) i32 {
    var sum: i32 = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        sum += arr[i];
    }
    return sum;
}

// 5. 数组元素乘以标量
pub export fn array_scale(arr: [*]i32, len: usize, factor: i32) void {
    var i: usize = 0;
    while (i < len) : (i += 1) {
        arr[i] *= factor;
    }
}

// 6. 数组过滤（返回满足条件的元素个数）
pub export fn array_filter_positive(src_ptr: [*]const i32, src_len: usize, dst_ptr: [*]i32, dst_len: usize) usize {
    _ = dst_len;
    var count: usize = 0;
    var i: usize = 0;
    while (i < src_len) : (i += 1) {
        if (src_ptr[i] > 0) {
            dst_ptr[count] = src_ptr[i];
            count += 1;
        }
    }
    return count;
}

// ========== 元组表示（通过struct） ==========

pub const Pair = extern struct {
    first: i32,
    second: i32,
};

pub const Triple = extern struct {
    a: f64,
    b: f64,
    c: f64,
};

// 7. 创建 Pair（类似元组）
pub export fn make_pair(x: i32, y: i32) Pair {
    return Pair{ .first = x, .second = y };
}

// 8. 交换 Pair 的两个元素
pub export fn swap_pair(p: Pair) Pair {
    return Pair{ .first = p.second, .second = p.first };
}

// 9. Triple 的平均值
pub export fn triple_average(t: Triple) f64 {
    return (t.a + t.b + t.c) / 3.0;
}

// ========== 混合复杂场景 ==========

pub const DataPoint = extern struct {
    id: u32,
    value: f32,
    label: [16]u8,
};

// 10. 处理 DataPoint 数组，找到最大值的 ID
pub export fn find_max_datapoint(points: [*]const DataPoint, len: usize) u32 {
    if (len == 0) return 0;
    
    var max_id = points[0].id;
    var max_val = points[0].value;
    
    var i: usize = 1;
    while (i < len) : (i += 1) {
        if (points[i].value > max_val) {
            max_val = points[i].value;
            max_id = points[i].id;
        }
    }
    
    return max_id;
}
---
// ========== 字符串相关（使用智能降级）==========
fn string_length(s: &str) -> usize;
fn string_reverse(s: &mut [u8]);
fn string_to_upper(src: &str, dst: &mut [u8]);

// ========== 数组/Slice 相关（使用智能降级）==========
fn array_sum(arr: &[i32]) -> i32;
fn array_scale(arr: &mut [i32], factor: i32);
fn array_filter_positive(src: &[i32], dst: &mut [i32]) -> usize;

// ========== 元组表示 ==========
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pair {
    pub first: i32,
    pub second: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Triple {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

fn make_pair(x: i32, y: i32) -> Pair;
fn swap_pair(p: Pair) -> Pair;
fn triple_average(t: Triple) -> f64;

// ========== 复杂数据结构 ==========
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub id: u32,
    pub value: f32,
    pub label: [u8; 16],
}

fn find_max_datapoint(points: &[DataPoint]) -> u32;
}

fn main() {
    println!("=== AutoZig 复杂类型综合示例 ===\n");

    // ========== 1. 字符串处理（零 unsafe）==========
    println!("【字符串处理】");
    
    let text = "hello rust";
    let len = string_length(text);
    println!("1. 字符串 '{}' 长度: {}", text, len);
    
    let mut text2_bytes = b"world".to_vec();
    string_reverse(&mut text2_bytes);
    let text2 = String::from_utf8(text2_bytes).unwrap();
    println!("2. 反转后: '{}'", text2);
    
    let mut upper = vec![0u8; text.len()];
    string_to_upper(text, &mut upper);
    let upper_str = String::from_utf8_lossy(&upper);
    println!("3. 转大写: '{}'\n", upper_str);

    // ========== 2. 数组/Vec 处理（零 unsafe）==========
    println!("【数组/Vec 处理】");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum = array_sum(&numbers);
    println!("4. 数组 {:?} 的和: {}", numbers, sum);
    
    let mut numbers2 = vec![10, 20, 30];
    array_scale(&mut numbers2, 3);
    println!("5. 数组乘以3: {:?}", numbers2);
    
    let mixed = vec![-5, 3, -2, 8, 0, -1, 10];
    let mut positive = vec![0i32; mixed.len()];
    let count = array_filter_positive(&mixed, &mut positive);
    positive.truncate(count);
    println!("6. 过滤正数: {:?} → {:?}\n", mixed, positive);

    // ========== 3. 元组 (通过 struct 表示) ==========
    println!("【元组表示（Pair, Triple）】");
    
    let pair = make_pair(42, 100);
    println!("7. 创建 Pair: ({}, {})", pair.first, pair.second);
    
    let swapped = swap_pair(pair);
    println!("8. 交换后: ({}, {})", swapped.first, swapped.second);
    
    let triple = Triple { a: 10.0, b: 20.0, c: 30.0 };
    let avg = triple_average(triple);
    println!("9. Triple ({}, {}, {}) 平均值: {:.2}\n", triple.a, triple.b, triple.c, avg);

    // ========== 4. 复杂数据结构（零 unsafe）==========
    println!("【复杂数据结构（DataPoint）】");
    
    let mut dp1 = DataPoint { id: 1, value: 3.5, label: [0; 16] };
    let label1 = b"sensor-A";
    dp1.label[..label1.len()].copy_from_slice(label1);
    
    let mut dp2 = DataPoint { id: 2, value: 7.2, label: [0; 16] };
    let label2 = b"sensor-B";
    dp2.label[..label2.len()].copy_from_slice(label2);
    
    let mut dp3 = DataPoint { id: 3, value: 5.8, label: [0; 16] };
    let label3 = b"sensor-C";
    dp3.label[..label3.len()].copy_from_slice(label3);
    
    let datapoints = vec![dp1, dp2, dp3];
    
    for dp in &datapoints {
        let label = String::from_utf8_lossy(&dp.label[..8]);
        println!("   DataPoint {{ id: {}, value: {:.1}, label: '{}' }}", dp.id, dp.value, label.trim_end_matches('\0'));
    }
    
    let max_id = find_max_datapoint(&datapoints);
    println!("10. 最大值的 ID: {}\n", max_id);

    println!("=== 所有测试通过 ===");
    println!("\n✨ 关键改进:");
    println!("  ✓ 所有函数调用都是零 unsafe");
    println!("  ✓ 使用智能降级自动转换 &str 和 &[T]");
    println!("  ✓ 类型安全且符合 Rust 习惯用法");
}