use autozig::autozig;

autozig! {
const std = @import("std");

// 定义 Result 类型的 C 表示（使用 tagged union）
pub const ResultTag = enum(u8) {
    Ok = 0,
    Err = 1,
};

pub const ResultInt = extern struct {
    tag: ResultTag,
    value: i32,  // Ok 时是结果值，Err 时是错误码
};

// 定义 Option 类型的 C 表示
pub const OptionTag = enum(u8) {
    None = 0,
    Some = 1,
};

pub const OptionInt = extern struct {
    tag: OptionTag,
    value: i32,  // Some 时有效
};

// 定义自定义枚举：Status
pub const Status = enum(u8) {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
};

// 返回 Result<i32, i32>：除法操作
pub export fn divide(a: i32, b: i32) ResultInt {
    if (b == 0) {
        return ResultInt{
            .tag = ResultTag.Err,
            .value = -1,  // 错误码：除零错误
        };
    }
    return ResultInt{
        .tag = ResultTag.Ok,
        .value = @divTrunc(a, b),
    };
}

// 返回 Option<i32>：查找数组中的最大值
pub export fn find_max(arr: [*]const i32, len: usize) OptionInt {
    if (len == 0) {
        return OptionInt{
            .tag = OptionTag.None,
            .value = 0,
        };
    }
    
    var max_val = arr[0];
    var i: usize = 1;
    while (i < len) : (i += 1) {
        if (arr[i] > max_val) {
            max_val = arr[i];
        }
    }
    
    return OptionInt{
        .tag = OptionTag.Some,
        .value = max_val,
    };
}

// 处理自定义枚举：根据 Status 返回描述
pub export fn status_to_code(status: Status) u8 {
    return @intFromEnum(status);
}

// 从 u8 创建 Status
pub export fn code_to_status(code: u8) Status {
    return @enumFromInt(code);
}

// 状态机转换
pub export fn next_status(current: Status) Status {
    return switch (current) {
        Status.Idle => Status.Running,
        Status.Running => Status.Paused,
        Status.Paused => Status.Running,
        Status.Stopped => Status.Idle,
    };
}
---
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResultTag {
    Ok = 0,
    Err = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ResultInt {
    pub tag: ResultTag,
    pub value: i32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionTag {
    None = 0,
    Some = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OptionInt {
    pub tag: OptionTag,
    pub value: i32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
}

fn divide(a: i32, b: i32) -> ResultInt;
fn find_max(arr: *const i32, len: usize) -> OptionInt;
fn status_to_code(status: Status) -> u8;
fn code_to_status(code: u8) -> Status;
fn next_status(current: Status) -> Status;
}

fn main() {
    println!("=== AutoZig Enum 示例 ===\n");

    // 测试 Result<i32, i32>
    println!("1. 测试 Result 类型（除法操作）:");
    
    let result1 = divide(10, 2);
    match result1.tag {
        ResultTag::Ok => println!("   10 / 2 = {} (成功)", result1.value),
        ResultTag::Err => println!("   错误码: {}", result1.value),
    }
    
    let result2 = divide(10, 0);
    match result2.tag {
        ResultTag::Ok => println!("   10 / 0 = {} (成功)", result2.value),
        ResultTag::Err => println!("   10 / 0 = 错误（除零），错误码: {}", result2.value),
    }

    // 测试 Option<i32>
    println!("\n2. 测试 Option 类型（查找最大值）:");
    
    let arr = vec![3, 7, 2, 9, 4];
    let option1 = find_max(arr.as_ptr(), arr.len());
    match option1.tag {
        OptionTag::Some => println!("   数组 {:?} 的最大值: {}", arr, option1.value),
        OptionTag::None => println!("   数组为空，无最大值"),
    }
    
    let empty: Vec<i32> = vec![];
    let option2 = find_max(empty.as_ptr(), empty.len());
    match option2.tag {
        OptionTag::Some => println!("   最大值: {}", option2.value),
        OptionTag::None => println!("   空数组，返回 None"),
    }

    // 测试自定义枚举 Status
    println!("\n3. 测试自定义枚举（Status 状态机）:");
    
    let status = Status::Idle;
    println!("   初始状态: {:?} (code: {})", status, status_to_code(status));
    
    let next = next_status(status);
    println!("   转换后: {:?} (code: {})", next, status_to_code(next));
    
    let next2 = next_status(next);
    println!("   再转换: {:?} (code: {})", next2, status_to_code(next2));
    
    let next3 = next_status(next2);
    println!("   再转换: {:?} (code: {})", next3, status_to_code(next3));

    // 测试 code 转 enum
    println!("\n4. 测试 u8 → Status 转换:");
    for code in 0..4 {
        let status = code_to_status(code);
        println!("   code {} → {:?}", code, status);
    }

    println!("\n=== 所有测试通过 ===");
}