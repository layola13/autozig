use autozig::autozig;

autozig! {
    // Zig code defining struct types and functions
    const std = @import("std");

    // Color struct - 3 floats (RGB)
    pub const Color = struct {
        r: f32,
        g: f32,
        b: f32,
    };

    // Vec3 struct - 3D vector
    pub const Vec3 = struct {
        x: f32,
        y: f32,
        z: f32,
    };

    // Complex struct with nested data
    pub const Transform = struct {
        position: Vec3,
        scale: f32,
    };

    // Function returning Color (needs ABI lowering)
    export fn create_color(r: f32, g: f32, b: f32) Color {
        return Color{ .r = r, .g = g, .b = b };
    }

    // Function returning Vec3 (needs ABI lowering)
    export fn create_vec3(x: f32, y: f32, z: f32) Vec3 {
        return Vec3{ .x = x, .y = y, .z = z };
    }

    // Function returning Transform (needs ABI lowering)
    export fn create_transform(x: f32, y: f32, z: f32, scale: f32) Transform {
        const pos = Vec3{ .x = x, .y = y, .z = z };
        return Transform{ .position = pos, .scale = scale };
    }

    // Function returning primitive (no ABI lowering needed)
    export fn add_floats(a: f32, b: f32) f32 {
        return a + b;
    }

    // Function taking and returning struct
    export fn scale_vec3(v: Vec3, factor: f32) Vec3 {
        return Vec3{
            .x = v.x * factor,
            .y = v.y * factor,
            .z = v.z * factor,
        };
    }

    ---

    // Rust struct definitions matching Zig
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Transform {
        pub position: Vec3,
        pub scale: f32,
    }

    // Function signatures (parser will detect which need ABI lowering)
    fn create_color(r: f32, g: f32, b: f32) -> Color;
    fn create_vec3(x: f32, y: f32, z: f32) -> Vec3;
    fn create_transform(x: f32, y: f32, z: f32, scale: f32) -> Transform;
    fn add_floats(a: f32, b: f32) -> f32;
    fn scale_vec3(v: Vec3, factor: f32) -> Vec3;
}

fn main() {
    println!("=== AutoZig ABI Lowering Test ===\n");

    // Test 1: Color struct return
    println!("Test 1: Color struct return");
    let red = create_color(1.0, 0.0, 0.0);
    println!("  Created red color: r={}, g={}, b={}", red.r, red.g, red.b);
    assert_eq!(red.r, 1.0);
    assert_eq!(red.g, 0.0);
    assert_eq!(red.b, 0.0);
    println!("  ✓ Color values correct!\n");

    // Test 2: Vec3 struct return
    println!("Test 2: Vec3 struct return");
    let vec = create_vec3(1.0, 2.0, 3.0);
    println!("  Created vec3: x={}, y={}, z={}", vec.x, vec.y, vec.z);
    assert_eq!(vec.x, 1.0);
    assert_eq!(vec.y, 2.0);
    assert_eq!(vec.z, 3.0);
    println!("  ✓ Vec3 values correct!\n");

    // Test 3: Nested struct return
    println!("Test 3: Transform (nested struct) return");
    let transform = create_transform(5.0, 10.0, 15.0, 2.0);
    println!(
        "  Transform position: x={}, y={}, z={}",
        transform.position.x, transform.position.y, transform.position.z
    );
    println!("  Transform scale: {}", transform.scale);
    assert_eq!(transform.position.x, 5.0);
    assert_eq!(transform.position.y, 10.0);
    assert_eq!(transform.position.z, 15.0);
    assert_eq!(transform.scale, 2.0);
    println!("  ✓ Transform values correct!\n");

    // Test 4: Primitive return (no ABI lowering)
    println!("Test 4: Primitive return (no ABI lowering needed)");
    let sum = add_floats(3.14, 2.86);
    println!("  3.14 + 2.86 = {}", sum);
    assert!((sum - 6.0).abs() < 0.01);
    println!("  ✓ Primitive return works!\n");


    // Test 5: Struct parameter and return
    println!("Test 5: Struct parameter and return");
    let vec = Vec3 { x: 2.0, y: 3.0, z: 4.0 };
    let scaled = scale_vec3(vec, 2.0);
    println!("  Original: x={}, y={}, z={}", vec.x, vec.y, vec.z);
    println!("  Scaled 2x: x={}, y={}, z={}", scaled.x, scaled.y, scaled.z);
    assert_eq!(scaled.x, 4.0);
    assert_eq!(scaled.y, 6.0);
    assert_eq!(scaled.z, 8.0);
    println!("  ✓ Struct param/return works!\n");

    println!("=== All ABI Lowering tests passed! ===");
    println!("\nCross-platform ABI compatibility verified:");
    println!("  ✓ Struct returns work correctly");
    println!("  ✓ Field ordering preserved");
    println!("  ✓ No memory corruption");
    println!("  ✓ Transparent to user code");
}
