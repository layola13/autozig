//! Array Parameter Test
//! 
//! This example demonstrates the automatic fixed-size array to pointer  
//! conversion feature in autozig. Users can declare [f32; 3] in Rust
//! and autozig automatically converts it to *const [3]f32 in the FFI.

use autozig::include_zig;

// Include Zig module with array parameter functions
// Notice: We use [f32; 3] syntax - autozig converts to *const [3]f32 automatically!
include_zig!("src/zig/array_ops.zig", {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
    
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Quat {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }
    
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Mat4 {
        pub data: [f32; 16],
    }

    // These use [f32; N] syntax - autozig auto-converts to pointers!
    fn vec3_add(a: [f32; 3], b: [f32; 3]) -> Vec3;
    fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32;
    fn vec3_from_array(arr: [f32; 3]) -> Vec3;
    
    fn quat_from_array(arr: [f32; 4]) -> Quat;
    fn quat_conjugate(q: [f32; 4]) -> Quat;
    
    fn mat4_identity() -> Mat4;
    fn mat4_from_array(arr: [f32; 16]) -> Mat4;
    fn mat4_get_translation(m: [f32; 16]) -> Vec3;
    fn mat4_scale(m: [f32; 16], s: f32, out: *mut [f32; 16]);
});

fn main() {
    println!("=== Array Parameter Auto-Conversion Test ===\n");
    
    // Test Vec3 operations
    println!("--- Vec3 Operations ---");
    let a: [f32; 3] = [1.0, 2.0, 3.0];
    let b: [f32; 3] = [4.0, 5.0, 6.0];
    
    let sum = vec3_add(a, b);
    println!("vec3_add({:?}, {:?}) = {:?}", a, b, sum);
    assert!((sum.x - 5.0).abs() < 0.001);
    assert!((sum.y - 7.0).abs() < 0.001);
    assert!((sum.z - 9.0).abs() < 0.001);
    
    let dot = vec3_dot(a, b);
    println!("vec3_dot({:?}, {:?}) = {}", a, b, dot);
    assert!((dot - 32.0).abs() < 0.001); // 1*4 + 2*5 + 3*6 = 32
    
    let v = vec3_from_array([10.0, 20.0, 30.0]);
    println!("vec3_from_array([10, 20, 30]) = {:?}", v);
    
    // Test Quaternion operations
    println!("\n--- Quaternion Operations ---");
    let q_arr: [f32; 4] = [0.0, 0.707, 0.0, 0.707];
    
    let q = quat_from_array(q_arr);
    println!("quat_from_array({:?}) = {:?}", q_arr, q);
    
    let q_conj = quat_conjugate(q_arr);
    println!("quat_conjugate({:?}) = {:?}", q_arr, q_conj);
    assert!((q_conj.x - 0.0).abs() < 0.001);
    assert!((q_conj.y - (-0.707)).abs() < 0.001);
    
    // Test Mat4 operations
    println!("\n--- Mat4 Operations ---");
    let identity = mat4_identity();
    println!("mat4_identity() = {:?}", identity);
    
    // Create a translation matrix
    let mut translation_matrix: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        5.0, 10.0, 15.0, 1.0,
    ];
    
    let m = mat4_from_array(translation_matrix);
    println!("mat4_from_array(translation_matrix) = {:?}", m);
    
    let trans = mat4_get_translation(translation_matrix);
    println!("mat4_get_translation(m) = {:?}", trans);
    assert!((trans.x - 5.0).abs() < 0.001);
    assert!((trans.y - 10.0).abs() < 0.001);
    assert!((trans.z - 15.0).abs() < 0.001);
    
    let mut scaled: [f32; 16] = [0.0; 16];
    mat4_scale(translation_matrix, 2.0, &mut scaled);
    println!("mat4_scale(m, 2.0) first 4 values = {:?}", &scaled[0..4]);
    assert!((scaled[0] - 2.0).abs() < 0.001);
    
    println!("\n=== All tests passed! ===");
    println!("✓ Fixed-size array parameters work correctly");
    println!("✓ Backward compatible with existing pointer types");
}
