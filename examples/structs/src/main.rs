use autozig::autozig;

autozig! {
const std = @import("std");

// 定义一个简单的 Point 结构体
pub const Point = extern struct {
    x: i32,
    y: i32,
};

// 定义一个包含多种类型的 Person 结构体
pub const Person = extern struct {
    age: u8,
    height: f32,
    is_student: bool,
};

// 定义一个嵌套结构体 Rectangle
pub const Rectangle = extern struct {
    top_left: Point,
    width: i32,
    height: i32,
};

// 计算点到原点的距离
pub export fn point_distance(p: Point) f64 {
    const x_f: f64 = @floatFromInt(p.x);
    const y_f: f64 = @floatFromInt(p.y);
    return @sqrt(x_f * x_f + y_f * y_f);
}

// 创建一个新的 Point
pub export fn point_new(x: i32, y: i32) Point {
    return Point{ .x = x, .y = y };
}

// 修改 Point 的坐标（通过指针）
pub export fn point_move(p: *Point, dx: i32, dy: i32) void {
    p.x += dx;
    p.y += dy;
}

// 创建一个新的 Person
pub export fn person_new(age: u8, height: f32, is_student: bool) Person {
    return Person{
        .age = age,
        .height = height,
        .is_student = is_student,
    };
}

// 计算矩形面积
pub export fn rect_area(r: Rectangle) i32 {
    return r.width * r.height;
}

// 创建一个新的 Rectangle
pub export fn rect_new(x: i32, y: i32, w: i32, h: i32) Rectangle {
    return Rectangle{
        .top_left = Point{ .x = x, .y = y },
        .width = w,
        .height = h,
    };
}
---
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Person {
    pub age: u8,
    pub height: f32,
    pub is_student: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub top_left: Point,
    pub width: i32,
    pub height: i32,
}

fn point_distance(p: Point) -> f64;
fn point_new(x: i32, y: i32) -> Point;
fn point_move(p: *mut Point, dx: i32, dy: i32) -> ();
fn person_new(age: u8, height: f32, is_student: u8) -> Person;
fn rect_area(r: Rectangle) -> i32;
fn rect_new(x: i32, y: i32, w: i32, h: i32) -> Rectangle;
}

fn main() {
    println!("=== AutoZig Struct 示例 ===\n");

    // 测试 Point
    println!("1. 测试 Point 结构体:");
    let p1 = point_new(3, 4);
    println!("   创建点: Point {{ x: {}, y: {} }}", p1.x, p1.y);

    let dist = point_distance(p1);
    println!("   到原点距离: {:.2}", dist);

    let mut p2 = point_new(10, 20);
    println!("   移动前: Point {{ x: {}, y: {} }}", p2.x, p2.y);
    point_move(&mut p2, 5, -3);
    println!("   移动后: Point {{ x: {}, y: {} }}", p2.x, p2.y);

    // 测试 Person
    println!("\n2. 测试 Person 结构体:");
    let person = person_new(25, 1.75, 1);
    println!("   创建人物: Person {{");
    println!("     age: {},", person.age);
    println!("     height: {:.2}m,", person.height);
    println!("     is_student: {}", person.is_student != 0);
    println!("   }}");

    // 测试 Rectangle（嵌套结构体）
    println!("\n3. 测试 Rectangle 嵌套结构体:");
    let rect = rect_new(0, 0, 100, 50);
    println!("   创建矩形: Rectangle {{");
    println!("     top_left: Point {{ x: {}, y: {} }},", rect.top_left.x, rect.top_left.y);
    println!("     width: {},", rect.width);
    println!("     height: {}", rect.height);
    println!("   }}");

    let area = rect_area(rect);
    println!("   矩形面积: {}", area);

    // 测试结构体克隆和复制
    println!("\n4. 测试结构体复制语义:");
    let p3 = p1; // Copy trait，这是值复制
    println!("   原始点: Point {{ x: {}, y: {} }}", p1.x, p1.y);
    println!("   复制点: Point {{ x: {}, y: {} }}", p3.x, p3.y);

    println!("\n=== 所有测试通过 ===");
}
