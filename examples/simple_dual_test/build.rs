fn main() {
    // 简单的构建脚本，使用 autozig-build
    autozig_build::build("src").expect("Build failed");
}