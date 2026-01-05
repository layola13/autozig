# CI/CD Configuration Guide

## Overview

AutoZig uses GitHub Actions for continuous integration and Git hooks for local development quality checks.

---

## GitHub Actions CI

### Workflows

The CI pipeline (`.github/workflows/ci.yml`) includes multiple jobs:

#### 1. **Test Suite** (`test`)
- **Matrix**: Ubuntu/macOS × Stable/Nightly Rust × Zig 0.11/0.12/0.13
- **Steps**:
  - Install Rust and Zig
  - Cache dependencies
  - Run `cargo test --all`
  - Run doc tests

#### 2. **Format Check** (`fmt`)
- **Platform**: Ubuntu latest
- **Check**: `cargo fmt --all -- --check`
- **Purpose**: Ensure code follows Rust formatting standards

#### 3. **Clippy Lints** (`clippy`)
- **Platform**: Ubuntu latest
- **Check**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Purpose**: Catch common mistakes and enforce best practices

#### 4. **Build** (`build`)
- **Matrix**: Ubuntu/macOS/Windows
- **Steps**:
  - Build release binary
  - Build all examples
- **Purpose**: Ensure cross-platform compatibility

#### 5. **Examples Verification** (`examples`)
- **Platform**: Ubuntu latest
- **Script**: `examples/verify_all.sh`
- **Purpose**: Verify all 10 examples work correctly

#### 6. **Security Audit** (`security-audit`)
- **Tool**: `cargo-audit`
- **Purpose**: Check for known security vulnerabilities in dependencies

#### 7. **Code Coverage** (`coverage`)
- **Tool**: `cargo-tarpaulin`
- **Upload**: Codecov
- **Purpose**: Track test coverage metrics

### Supported Versions

| Component | Versions |
|-----------|----------|
| **Rust** | stable, nightly |
| **Zig** | 0.11.0, 0.12.0, 0.13.0 |
| **OS** | Ubuntu, macOS, Windows |

### Status Badges

Add these to your README.md:

```markdown
[![CI](https://github.com/yourusername/autozig/workflows/CI/badge.svg)](https://github.com/yourusername/autozig/actions)
[![codecov](https://codecov.io/gh/yourusername/autozig/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/autozig)
```

---

## Git Hooks

### Pre-Push Hook

Located at `.githooks/pre-push`, this hook runs automatically before every `git push`.

#### Checks Performed

1. **Format Check** (`cargo fmt --all -- --check`)
2. **Clippy Lints** (`cargo clippy --all-targets --all-features`)
3. **Build** (`cargo build --all`)
4. **Tests** (`cargo test --all`)
5. **Doc Tests** (`cargo test --doc --all`)
6. **Examples Verification** (`examples/verify_all.sh`) *(optional)*

### Installation

Run the installation script:

```bash
cd autozig
./scripts/install-hooks.sh
```

This creates symlinks from `.git/hooks/` to `.githooks/`.

### Usage

#### Normal Push
```bash
git push
# Hook runs automatically
```

#### Skip Examples Verification
If examples take too long:
```bash
SKIP_EXAMPLES=1 git push
```

#### Bypass Hook (Not Recommended)
```bash
git push --no-verify
```

### Uninstall

```bash
rm autozig/.git/hooks/pre-push
```

---

## Local Development Workflow

### Before Committing

1. **Format code**:
   ```bash
   cargo fmt --all
   ```

2. **Fix clippy warnings**:
   ```bash
   cargo clippy --all-targets --all-features --fix
   ```

3. **Run tests**:
   ```bash
   cargo test --all
   ```

### Before Pushing

The pre-push hook will automatically run all checks. If you want to run them manually:

```bash
cd autozig

# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --all

# Tests
cargo test --all

# Examples
cd examples && ./verify_all.sh
```

---

## Continuous Deployment (Future)

### Planned Features

- **Auto-release**: Create GitHub releases on version tags
- **Crates.io publishing**: Automatic publishing to crates.io
- **Documentation**: Deploy docs to GitHub Pages
- **Benchmarks**: Performance regression tracking

### Release Process (Manual for now)

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create git tag: `git tag -a v0.2.0 -m "Release v0.2.0"`
4. Push tag: `git push origin v0.2.0`
5. Publish to crates.io:
   ```bash
   cd autozig/parser && cargo publish
   cd ../macro && cargo publish
   cd ../engine && cargo publish
   cd ../gen/build && cargo publish
   cd ../.. && cargo publish
   ```

---

## Troubleshooting

### CI Fails on Zig Installation

**Problem**: Zig version not available for the platform

**Solution**: Update `.github/workflows/ci.yml` to use compatible versions

### Pre-Push Hook Too Slow

**Problem**: Examples verification takes >5 minutes

**Solutions**:
1. Skip examples: `SKIP_EXAMPLES=1 git push`
2. Reduce timeout in hook script
3. Run examples in parallel

### Clippy False Positives

**Problem**: Clippy warns about intentional code patterns

**Solution**: Add `#[allow(clippy::...)]` attribute:
```rust
#[allow(clippy::too_many_arguments)]
fn complex_function(...) { }
```

### Format Conflicts

**Problem**: `cargo fmt` changes differ between Rust versions

**Solution**: Lock to specific Rust version in `rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.77.0"
```

---

## Best Practices

### For Contributors

1. ✅ **Always install hooks**: Run `./scripts/install-hooks.sh` after cloning
2. ✅ **Format before commit**: `cargo fmt --all`
3. ✅ **Fix warnings**: Don't ignore clippy warnings
4. ✅ **Test locally**: Run full test suite before pushing
5. ✅ **Keep CI green**: Fix failures immediately

### For Maintainers

1. ✅ **Review CI logs**: Check all jobs pass before merging PRs
2. ✅ **Monitor coverage**: Aim for >80% test coverage
3. ✅ **Update dependencies**: Run `cargo update` regularly
4. ✅ **Security audits**: Check `cargo audit` monthly
5. ✅ **Performance**: Track benchmark results

---

## Configuration Files

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | GitHub Actions CI pipeline |
| `.githooks/pre-push` | Pre-push validation hook |
| `scripts/install-hooks.sh` | Hook installation script |
| `examples/verify_all.sh` | Batch example verification |
| `rustfmt.toml` | Code formatting rules |
| `.clippy.toml` | Clippy configuration |

---

## Metrics

### Current Status

- **Test Suite**: 35 tests passing
- **Examples**: 10 examples verified
- **Platforms**: Linux, macOS, Windows
- **Rust Versions**: stable, nightly
- **Zig Versions**: 0.11, 0.12, 0.13

### CI Performance

| Job | Average Duration |
|-----|------------------|
| Test | ~2-3 minutes |
| Format | ~30 seconds |
| Clippy | ~1-2 minutes |
| Build | ~3-4 minutes |
| Examples | ~5-8 minutes |
| Coverage | ~4-5 minutes |

---

## Future Improvements

- [ ] Add benchmark tracking
- [ ] Implement auto-release workflow
- [ ] Add PR size checker
- [ ] Implement dependency update automation
- [ ] Add commit message linter
- [ ] Create nightly build workflow
- [ ] Add performance regression tests

---

## Support

If you encounter CI/CD issues:

1. Check workflow logs in GitHub Actions tab
2. Run checks locally to reproduce
3. Review this documentation
4. Open an issue with error logs

---

**Last Updated**: 2026-01-05  
**CI/CD Version**: 1.0.0