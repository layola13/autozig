//! SIMD Optimization Detection
//!
//! This module detects CPU features at compile-time and configures Zig
//! compilation to use appropriate SIMD instructions.

use std::env;

/// CPU Architecture detected from Rust target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    X86_64,
    Aarch64,
    Arm,
    Other,
}

/// SIMD feature sets available on x86_64
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86SimdLevel {
    /// Baseline x86_64 (SSE2)
    Baseline,
    /// SSE4.2 support
    SSE4_2,
    /// AVX support
    AVX,
    /// AVX2 support
    AVX2,
    /// AVX-512 support
    AVX512,
}

/// SIMD feature sets available on ARM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmSimdLevel {
    /// No SIMD
    None,
    /// NEON support
    NEON,
    /// SVE support (Scalable Vector Extension)
    SVE,
}

/// SIMD configuration for Zig compilation
#[derive(Debug, Clone)]
pub struct SimdConfig {
    /// Target architecture
    pub arch: CpuArch,
    /// Zig CPU flag to pass to compiler
    pub zig_cpu_flag: String,
    /// Human-readable description
    pub description: String,
    /// Whether native CPU features should be used
    pub use_native: bool,
}

impl SimdConfig {
    /// Detect SIMD configuration from Rust environment
    pub fn detect() -> Self {
        let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
        let rust_flags = env::var("RUSTFLAGS").unwrap_or_default();
        let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();

        // Check if native CPU optimization is requested
        let use_native = rust_flags.contains("target-cpu=native")
            || rust_flags.contains("-C native")
            || rust_flags.contains("--target-cpu=native");

        let arch = Self::detect_arch(&target);

        let (zig_cpu_flag, description) = if use_native {
            // Use native CPU features
            ("-mcpu=native".to_string(), format!("{:?} with native CPU features", arch))
        } else {
            // Detect explicit features
            Self::detect_features(&target, &target_features, arch)
        };

        SimdConfig {
            arch,
            zig_cpu_flag,
            description,
            use_native,
        }
    }

    /// Detect CPU architecture from target triple
    fn detect_arch(target: &str) -> CpuArch {
        if target.contains("x86_64") {
            CpuArch::X86_64
        } else if target.contains("aarch64") {
            CpuArch::Aarch64
        } else if target.contains("arm") {
            CpuArch::Arm
        } else {
            CpuArch::Other
        }
    }

    /// Detect SIMD features from target and cargo configuration
    fn detect_features(target: &str, features: &str, arch: CpuArch) -> (String, String) {
        match arch {
            CpuArch::X86_64 => Self::detect_x86_features(target, features),
            CpuArch::Aarch64 | CpuArch::Arm => Self::detect_arm_features(features),
            CpuArch::Other => {
                ("-mcpu=baseline".to_string(), "baseline (unknown architecture)".to_string())
            },
        }
    }

    /// Detect x86_64 SIMD features
    fn detect_x86_features(target: &str, features: &str) -> (String, String) {
        // Check for AVX-512 first (most advanced)
        if features.contains("avx512") {
            return ("-mcpu=x86_64_v4".to_string(), "x86_64 v4 (AVX-512)".to_string());
        }

        // Check for AVX2
        if features.contains("avx2") {
            return ("-mcpu=x86_64_v3".to_string(), "x86_64 v3 (AVX2, FMA)".to_string());
        }

        // Check for AVX
        if features.contains("avx") {
            return ("-mcpu=x86_64+avx".to_string(), "x86_64 with AVX".to_string());
        }

        // Check for SSE4.2
        if features.contains("sse4.2") || features.contains("sse4_2") {
            return ("-mcpu=x86_64+sse4.2".to_string(), "x86_64 with SSE4.2".to_string());
        }

        // Default x86_64 baseline (SSE2)
        // x86_64 v1 is the baseline with SSE2
        if target.contains("x86_64") {
            ("-mcpu=x86_64".to_string(), "x86_64 baseline (SSE2)".to_string())
        } else {
            ("-mcpu=baseline".to_string(), "baseline".to_string())
        }
    }

    /// Detect ARM SIMD features
    fn detect_arm_features(features: &str) -> (String, String) {
        // Check for SVE
        if features.contains("sve") {
            return ("-mcpu=generic+sve".to_string(), "ARM with SVE".to_string());
        }

        // Check for NEON
        if features.contains("neon") {
            return ("-mcpu=generic+neon".to_string(), "ARM with NEON".to_string());
        }

        // ARM baseline
        ("-mcpu=generic".to_string(), "ARM generic".to_string())
    }

    /// Generate a compiler report for display during build
    pub fn report(&self) -> String {
        format!(
            r#"
╔════════════════════════════════════════════╗
║     AutoZig SIMD Optimization Report      ║
╚════════════════════════════════════════════╝

Architecture:    {:?}
Configuration:   {}
Zig CPU Flag:    {}
Native Features: {}

TIP: Set RUSTFLAGS="-C target-cpu=native" for maximum performance
"#,
            self.arch,
            self.description,
            self.zig_cpu_flag,
            if self.use_native {
                "ENABLED"
            } else {
                "disabled"
            }
        )
    }

    /// Get the Zig compiler flag as a string
    pub fn as_zig_flag(&self) -> &str {
        &self.zig_cpu_flag
    }
}

/// Detect and print SIMD configuration
///
/// This should be called from build.rs to configure Zig compilation
pub fn detect_and_report() -> SimdConfig {
    let config = SimdConfig::detect();

    // Print report to cargo output
    println!("cargo:warning={}", config.report());

    // Emit configuration for use in source code
    println!("cargo:rustc-env=AUTOZIG_SIMD_ARCH={:?}", config.arch);
    println!("cargo:rustc-env=AUTOZIG_SIMD_CONFIG={}", config.description);
    println!("cargo:rustc-env=AUTOZIG_ZIG_CPU_FLAG={}", config.zig_cpu_flag);

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_arch() {
        assert_eq!(SimdConfig::detect_arch("x86_64-unknown-linux-gnu"), CpuArch::X86_64);
        assert_eq!(SimdConfig::detect_arch("aarch64-apple-darwin"), CpuArch::Aarch64);
        assert_eq!(SimdConfig::detect_arch("armv7-unknown-linux"), CpuArch::Arm);
    }

    #[test]
    fn test_x86_feature_detection() {
        let (flag, desc) = SimdConfig::detect_x86_features("x86_64-pc-windows-msvc", "avx2,fma");
        assert!(flag.contains("x86_64"));
        assert!(desc.contains("AVX2") || desc.contains("v3"));

        let (flag, desc) = SimdConfig::detect_x86_features("x86_64-pc-windows-msvc", "sse4.2");
        assert!(flag.contains("sse4.2") || flag.contains("x86_64"));
        assert!(desc.contains("SSE") || desc.contains("baseline"));
    }

    #[test]
    fn test_arm_feature_detection() {
        let (flag, desc) = SimdConfig::detect_arm_features("neon");
        assert!(flag.contains("neon") || flag.contains("generic"));
        assert!(desc.contains("NEON") || desc.contains("ARM"));
    }

    #[test]
    fn test_config_detection() {
        // Just ensure it doesn't panic
        let config = SimdConfig::detect();
        assert!(!config.zig_cpu_flag.is_empty());
        assert!(!config.description.is_empty());
    }

    #[test]
    fn test_report_generation() {
        let config = SimdConfig::detect();
        let report = config.report();
        assert!(report.contains("AutoZig SIMD"));
        assert!(report.contains("Architecture"));
    }
}
