//! Zig compiler wrapper with target support

use std::{
    path::Path,
    process::Command,
};

use anyhow::{
    Context,
    Result,
};

/// Wrapper for invoking the Zig compiler
pub struct ZigCompiler {
    zig_path: String,
}

impl ZigCompiler {
    /// Create a new Zig compiler wrapper
    pub fn new() -> Self {
        // Check for ZIG_PATH environment variable, otherwise use "zig"
        let zig_path = std::env::var("ZIG_PATH").unwrap_or_else(|_| "zig".to_string());
        Self { zig_path }
    }

    /// Check Zig compiler version
    pub fn check_version(&self) -> Result<String> {
        let output = Command::new(&self.zig_path)
            .arg("version")
            .output()
            .context("Failed to execute zig version command")?;

        if !output.status.success() {
            anyhow::bail!("Zig compiler not found or failed to run");
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Compile Zig source to static library with target support
    ///
    /// # Arguments
    /// * `source` - Path to .zig source file
    /// * `output_lib` - Path for output static library (.a)
    /// * `target` - Target triple (e.g., "x86_64-linux-gnu", "native",
    ///   "wasm32-freestanding")
    pub fn compile_with_target(
        &self,
        source: &Path,
        output_lib: &Path,
        target: &str,
    ) -> Result<()> {
        println!("cargo:warning=Compiling Zig code: {} for target: {}", source.display(), target);

        // 查找同目录下的所有 C 源文件
        let c_sources = self.find_c_sources(source)?;

        if !c_sources.is_empty() {
            println!(
                "cargo:warning=Found {} C source file(s) to compile with Zig",
                c_sources.len()
            );
            for c_file in &c_sources {
                println!("cargo:warning=  - {}", c_file.display());
            }
        }

        // 检测是否为 WASM 目标
        let is_wasm = target.contains("wasm32");

        // zig build-lib source.zig -static -femit-bin=output.a -target <target>
        let mut cmd = Command::new(&self.zig_path);
        cmd.arg("build-lib")
            .arg(source)
            .arg("-static")
            .arg(format!("-femit-bin={}", output_lib.display()))
            .arg("-target")
            .arg(target);

        if is_wasm {
            // WASM 特殊配置
            println!("cargo:warning=Detected WASM target, applying WASM-specific flags");

            // WASM 不需要栈保护（没有 OS 支持）
            cmd.arg("-fno-stack-protector");

            // 🚀 启用 WASM SIMD128 支持（关键性能优化！）
            // 这将允许使用 v128.load, v128.sub, v128.store 等 SIMD 指令
            cmd.arg("-mcpu=mvp+simd128");

            // WASM 优化：改用 ReleaseFast 以获得最佳性能
            // (ReleaseSmall 会禁用某些 SIMD 优化)
            cmd.arg("-O").arg("ReleaseFast");

            // 不链接 libc（freestanding 环境）
            // WASM 环境下没有标准的 libc
        } else {
            // 非 WASM 目标的标准配置
            // Generate Position Independent Code (required for PIE executables)
            cmd.arg("-fPIC");

            // Link with libc (required for c_allocator and other libc functions)
            cmd.arg("-lc");

            // Optimize for release builds
            cmd.arg("-O").arg("ReleaseFast");
        }

        // 添加所有 C 源文件到编译命令（WASM 也支持 C 文件）
        for c_file in &c_sources {
            cmd.arg(c_file);
        }

        let status = cmd.status().context("Failed to execute zig build-lib")?;

        if !status.success() {
            anyhow::bail!("Zig compilation failed");
        }

        println!("cargo:warning=Zig compilation successful");
        println!("cargo:warning=Library: {}", output_lib.display());

        Ok(())
    }

    /// Compile with target and search for C sources in the provided src
    /// directory
    ///
    /// # Arguments
    /// * `source` - Path to .zig source file (usually in OUT_DIR)
    /// * `output_lib` - Path for output static library (.a)
    /// * `target` - Target triple (e.g., "x86_64-linux-gnu", "native",
    ///   "wasm32-freestanding")
    /// * `src_dir` - Original source directory to search for C files
    pub fn compile_with_target_and_src(
        &self,
        source: &Path,
        output_lib: &Path,
        target: &str,
        src_dir: &Path,
    ) -> Result<()> {
        println!("cargo:warning=Compiling Zig code: {} for target: {}", source.display(), target);

        // 在原始源码目录查找 C 源文件
        let c_sources = self.find_c_sources_in_dir(src_dir)?;

        if !c_sources.is_empty() {
            println!(
                "cargo:warning=Found {} C source file(s) to compile with Zig",
                c_sources.len()
            );
            for c_file in &c_sources {
                println!("cargo:warning=  - {}", c_file.display());
            }
        }

        // 检测是否为 WASM 目标
        let is_wasm = target.contains("wasm32");

        let mut cmd = Command::new(&self.zig_path);
        cmd.arg("build-lib")
            .arg(source)
            .arg("-static")
            .arg(format!("-femit-bin={}", output_lib.display()))
            .arg("-target")
            .arg(target);

        if is_wasm {
            // WASM 特殊配置
            cmd.arg("-fno-stack-protector")
                // 🚀 启用 WASM SIMD128 支持
                .arg("-mcpu=mvp+simd128")
                .arg("-O")
                .arg("ReleaseFast");
        } else {
            // 非 WASM 目标的标准配置
            cmd.arg("-fPIC").arg("-lc").arg("-O").arg("ReleaseFast");
        }

        // 添加所有 C 源文件到编译命令
        for c_file in &c_sources {
            cmd.arg(c_file);
        }

        let status = cmd.status().context("Failed to execute zig build-lib")?;

        if !status.success() {
            anyhow::bail!("Zig compilation failed");
        }

        println!("cargo:warning=Zig compilation successful");
        println!("cargo:warning=Library: {}", output_lib.display());

        Ok(())
    }

    /// Find all C source files in a specific directory
    fn find_c_sources_in_dir(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut c_sources = Vec::new();

        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "c" {
                            c_sources.push(path);
                        }
                    }
                }
            }
        }

        Ok(c_sources)
    }

    /// Find all C source files in the same directory as the Zig source
    fn find_c_sources(&self, zig_source: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut c_sources = Vec::new();

        if let Some(parent) = zig_source.parent() {
            if parent.exists() {
                for entry in std::fs::read_dir(parent)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "c" {
                                c_sources.push(path);
                            }
                        }
                    }
                }
            }
        }

        Ok(c_sources)
    }

    /// Compile with native target (convenience method)
    pub fn compile(&self, source: &Path, output_lib: &Path) -> Result<()> {
        self.compile_with_target(source, output_lib, "native")
    }

    /// Compile Zig tests to an executable
    ///
    /// # Arguments
    /// * `source` - Path to .zig source file containing tests
    /// * `output_exe` - Path for output test executable
    /// * `target` - Target triple (e.g., "x86_64-linux-gnu", "native")
    pub fn compile_tests(&self, source: &Path, output_exe: &Path, target: &str) -> Result<()> {
        println!("cargo:warning=Compiling Zig tests: {} for target: {}", source.display(), target);

        // zig test source.zig -femit-bin=output_exe -target <target>
        let status = Command::new(&self.zig_path)
            .arg("test")
            .arg(source)
            .arg(format!("-femit-bin={}", output_exe.display()))
            .arg("-target")
            .arg(target)
            // Optimize for release builds
            .arg("-O")
            .arg("ReleaseFast")
            .status()
            .context("Failed to execute zig test")?;

        if !status.success() {
            anyhow::bail!("Zig test compilation failed");
        }

        println!("cargo:warning=Zig test compilation successful");
        println!("cargo:warning=Test executable: {}", output_exe.display());

        Ok(())
    }

    /// Run compiled Zig test executable
    ///
    /// # Arguments
    /// * `test_exe` - Path to compiled test executable
    pub fn run_test_executable(&self, test_exe: &Path) -> Result<String> {
        let output = Command::new(test_exe)
            .output()
            .context(format!("Failed to execute test: {}", test_exe.display()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            anyhow::bail!("Zig tests failed:\nStdout: {}\nStderr: {}", stdout, stderr);
        }

        Ok(format!("Stdout: {}\nStderr: {}", stdout, stderr))
    }

    /// Compile using build.zig file
    ///
    /// # Arguments
    /// * `build_file` - Path to build.zig file
    /// * `build_dir` - Build directory (working directory for zig build)
    /// * `output_lib` - Expected output library path
    pub fn compile_with_buildzig(
        &self,
        build_file: &Path,
        build_dir: &Path,
        output_lib: &Path,
    ) -> Result<()> {
        println!("cargo:warning=Compiling with build.zig: {}", build_file.display());

        // Run: zig build --prefix-lib-dir <build_dir> --prefix <build_dir>
        let mut cmd = Command::new(&self.zig_path);
        cmd.arg("build")
            .arg("--build-file")
            .arg(build_file)
            .arg("--prefix")
            .arg(build_dir)
            .current_dir(build_dir);

        println!("cargo:warning=Running: {:?}", cmd);

        let output = cmd.output().context("Failed to execute zig build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Zig build failed:\nStdout: {}\nStderr: {}", stdout, stderr);
        }

        // The output library should be in build_dir/zig-out/lib/libautozig.a (Zig
        // 0.15.2+) Try multiple possible locations in order
        let possible_paths = vec![
            build_dir.join("zig-out").join("lib").join("libautozig.a"), // Zig 0.15.2+
            build_dir.join("lib").join("libautozig.a"),                 // Older Zig
            build_dir.join("libautozig.a"),                             // Direct output
        ];

        let mut found = false;
        for built_lib in &possible_paths {
            if built_lib.exists() {
                if built_lib != output_lib {
                    std::fs::copy(built_lib, output_lib).with_context(|| {
                        format!("Failed to copy built library from {}", built_lib.display())
                    })?;
                }
                found = true;
                break;
            }
        }

        if !found {
            anyhow::bail!(
                "Built library not found in any of these locations:\n  {}",
                possible_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }

        println!("cargo:warning=Build.zig compilation successful");
        println!("cargo:warning=Library: {}", output_lib.display());

        Ok(())
    }
}

impl Default for ZigCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = ZigCompiler::new();
        assert!(!compiler.zig_path.is_empty());
    }

    #[test]
    #[ignore] // Only run if Zig is installed
    fn test_check_version() {
        let compiler = ZigCompiler::new();
        let version = compiler.check_version();
        if version.is_ok() {
            println!("Zig version: {}", version.unwrap());
        }
    }
}
