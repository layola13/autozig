use autozig_engine::AutoZigEngine;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let src_dir = format!("{}/src", manifest_dir);
    
    let engine = AutoZigEngine::new(&src_dir, &out_dir);
    engine.build().expect("Failed to build Zig code");
    
    println!("cargo:rerun-if-changed=src/main.rs");
}