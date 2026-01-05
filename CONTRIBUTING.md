# Contributing to AutoZig

Thank you for your interest in contributing to AutoZig! This document provides guidelines and instructions for contributors.

---

## Getting Started

### Prerequisites

- **Rust**: 1.77+ (install via [rustup](https://rustup.rs/))
- **Zig**: 0.11+ (must be in PATH)
- **Git**: For version control

### Setup

1. **Fork and clone**:
   ```bash
   git clone https://github.com/yourusername/autozig.git
   cd autozig
   ```

2. **Install Git hooks**:
   ```bash
   ./scripts/install-hooks.sh
   ```

3. **Build and test**:
   ```bash
   cd autozig
   cargo build --all
   cargo test --all
   ```

---

## Development Workflow

### Before Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Ensure all tests pass:
   ```bash
   cargo test --all
   ```

### While Developing

1. **Write tests** for new functionality
2. **Update documentation** for API changes
3. **Follow code style**:
   ```bash
   cargo fmt --all
   ```

4. **Fix clippy warnings**:
   ```bash
   cargo clippy --all-targets --all-features --fix
   ```

### Before Committing

Run the pre-push checks manually:
```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test --all

# Examples
cd examples && ./verify_all.sh
```

### Committing

Use clear, descriptive commit messages:
```bash
git commit -m "feat: add generic monomorphization support"
git commit -m "fix: resolve memory leak in async wrapper"
git commit -m "docs: update README with Phase 3 features"
```

**Commit message prefixes**:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Build/tooling changes

### Pushing Changes

The pre-push hook will run automatically. If it passes:
```bash
git push origin feature/your-feature-name
```

To skip examples verification (faster):
```bash
SKIP_EXAMPLES=1 git push origin feature/your-feature-name
```

---

## Pull Request Process

### Creating a PR

1. Push your changes to your fork
2. Open a PR against the `main` branch
3. Fill out the PR template
4. Link any related issues

### PR Checklist

- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --all-targets`)
- [ ] Documentation is updated
- [ ] Examples work if applicable
- [ ] Commit messages are clear
- [ ] PR description explains the change

### Review Process

1. **Automated checks**: CI must pass
2. **Code review**: At least one maintainer approval
3. **Testing**: Verify examples if applicable
4. **Merge**: Squash and merge to main

---

## Code Style Guidelines

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with project configuration
- Address all `clippy` warnings
- Maximum line length: 100 characters
- Use meaningful variable names

**Example**:
```rust
// Good
pub fn generate_monomorphized_ffi(
    signature: &RustFunctionSignature,
    concrete_type: &str,
) -> TokenStream {
    // Implementation
}

// Bad
pub fn gen_mono(sig: &RustFunctionSignature, t: &str) -> TokenStream {
    // Implementation
}
```

### Zig Code

- Follow Zig style guide
- Use 4-space indentation
- Export functions with `export` keyword
- Add comments for complex logic

**Example**:
```zig
/// Computes sum of array elements
export fn sum_i32(data_ptr: [*]const i32, data_len: usize) i32 {
    var total: i32 = 0;
    var i: usize = 0;
    while (i < data_len) : (i += 1) {
        total += data_ptr[i];
    }
    return total;
}
```

### Documentation

- Add doc comments to all public APIs
- Include examples in doc comments
- Update relevant markdown files
- Keep README.md current

**Example**:
```rust
/// Generates FFI declarations for monomorphized generic functions.
///
/// # Arguments
///
/// * `signature` - The generic function signature
/// * `types` - Concrete types to instantiate
///
/// # Examples
///
/// ```rust
/// let signature = parse_signature("fn sum<T>(data: &[T]) -> T");
/// let types = vec!["i32", "f64"];
/// let code = generate_monomorphized_ffi(&signature, &types);
/// ```
pub fn generate_monomorphized_ffi(
    signature: &RustFunctionSignature,
    types: &[String],
) -> TokenStream {
    // ...
}
```

---

## Testing Guidelines

### Unit Tests

Place tests in the same file as the code:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monomorphization() {
        let input = /* ... */;
        let output = generate_monomorphized_ffi(/* ... */);
        assert!(/* ... */);
    }
}
```

### Integration Tests

Add integration tests in `tests/` directory:
```rust
// tests/generics_integration.rs
#[test]
fn test_generic_sum_i32() {
    let result = sum_i32(&[1, 2, 3, 4, 5]);
    assert_eq!(result, 15);
}
```

### Example Tests

Verify examples work:
```bash
cd examples/generics
cargo run --release
```

---

## Adding New Features

### For Major Features (Phase 4+)

1. Create a design document in `docs/`
2. Discuss design in an issue
3. Break work into smaller PRs
4. Update roadmap documentation

### For Minor Features

1. Open an issue describing the feature
2. Get maintainer approval
3. Implement with tests
4. Submit PR

---

## Reporting Issues

### Bug Reports

Include:
- **Description**: What went wrong?
- **Steps to reproduce**: How to trigger the bug?
- **Expected behavior**: What should happen?
- **Actual behavior**: What actually happened?
- **Environment**: OS, Rust version, Zig version
- **Code sample**: Minimal reproduction

### Feature Requests

Include:
- **Use case**: Why is this needed?
- **Proposed solution**: How should it work?
- **Alternatives**: Other approaches considered?
- **Examples**: Sample code showing desired API

---

## Project Structure

```
autozig/
├── src/              # Main library
├── parser/           # AST parsing
├── macro/            # Procedural macros
├── engine/           # Build engine
├── gen/build/        # Build helpers
├── examples/         # Example projects
├── docs/             # Documentation
├── scripts/          # Utility scripts
├── .github/          # CI configuration
└── .githooks/        # Git hooks
```

---

## CI/CD

### GitHub Actions

All PRs must pass CI checks:
- Tests on multiple platforms
- Clippy lints
- Format check
- Build verification
- Examples verification

See [docs/CI_CD.md](docs/CI_CD.md) for details.

### Pre-Push Hook

Automatically runs before `git push`:
- Format check
- Clippy
- Build
- Tests
- Examples (optional)

---

## Communication

- **GitHub Issues**: Bug reports, feature requests
- **Pull Requests**: Code contributions
- **Discussions**: Design discussions, Q&A

---

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

---

## Recognition

Contributors are recognized in:
- Git commit history
- Release notes
- CONTRIBUTORS.md (coming soon)

---

## Questions?

If you have questions about contributing:
1. Check existing documentation
2. Search closed issues
3. Open a new discussion
4. Ask in your PR

Thank you for contributing to AutoZig! 🎉