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
    /// * `target` - Target triple (e.g., "x86_64-linux-gnu", "native")
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

        // zig build-lib source.zig -static -femit-bin=output.a -target <target> -fPIC
        // -lc NOTE: We removed -femit-h because it's experimental and unstable
        // FFI bindings will be generated directly from Rust signatures (IDL-driven)
        // -fPIC is required for linking with PIE executables (Rust default)
        // -lc is required for linking with libc (needed for c_allocator)
        let mut cmd = Command::new(&self.zig_path);
        cmd.arg("build-lib")
            .arg(source)
            .arg("-static")
            .arg(format!("-femit-bin={}", output_lib.display()))
            .arg("-target")
            .arg(target)
            // Generate Position Independent Code (required for PIE executables)
            .arg("-fPIC")
            // Link with libc (required for c_allocator and other libc functions)
            .arg("-lc")
            // Optimize for release builds
            .arg("-O")
            .arg("ReleaseFast");

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

    /// Compile with target and search for C sources in the provided src
    /// directory
    ///
    /// # Arguments
    /// * `source` - Path to .zig source file (usually in OUT_DIR)
    /// * `output_lib` - Path for output static library (.a)
    /// * `target` - Target triple (e.g., "x86_64-linux-gnu", "native")
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

        let mut cmd = Command::new(&self.zig_path);
        cmd.arg("build-lib")
            .arg(source)
            .arg("-static")
            .arg(format!("-femit-bin={}", output_lib.display()))
            .arg("-target")
            .arg(target)
            .arg("-fPIC")
            .arg("-lc")
            .arg("-O")
            .arg("ReleaseFast");

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
