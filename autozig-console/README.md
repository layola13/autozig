# AutoZig Console

> Console logging support for AutoZig WASM applications

[![Crates.io](https://img.shields.io/crates/v/autozig-console.svg)](https://crates.io/crates/autozig-console)
[![Documentation](https://docs.rs/autozig-console/badge.svg)](https://docs.rs/autozig-console)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**AutoZig Console** provides `console_log!` and `console_error!` macros for WebAssembly applications, solving the problem of Rust's standard `print!` and `println!` macros being ineffective in browsers.

## ✨ Features

- ✅ **Rust → Zig → JS** three-layer architecture
- ✅ **WASM64 BigInt** pointer support (64-bit addressing)
- ✅ **Zero-copy** string passing (direct memory access)
- ✅ **Automatic panic hook** integration
- ✅ **Type-safe** FFI (no unsafe code needed by users)
- ✅ **No wasm-bindgen** required
- ✅ Works with both **WASM32** and **WASM64** targets

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
autozig-console = "0.1"
```

### Use in Your Code

```rust
use autozig_console::{console_log, console_error, init_panic_hook};

#[no_mangle]
pub extern "C" fn main() {
    // Initialize panic hook (optional but recommended)
    init_panic_hook();
    
    // Use console_log just like println!
    console_log!("Hello from WASM!");
    console_log!("Value: {}", 42);
    console_log!("Data: {:?}", vec![1, 2, 3]);
    
    // Use console_error for errors
    console_error!("Error: Something went wrong!");
}
```

### JavaScript Setup

Your JavaScript loader needs to provide the required functions:

```javascript
const memory = new WebAssembly.Memory({
    initial: 10,
    maximum: 100,
    index: 'i64'  // For WASM64 (use 'i32' for WASM32)
});

const imports = {
    env: {
        memory: memory,
        
        js_log: (ptrBigInt, lenBigInt) => {
            const ptr = Number(ptrBigInt);
            const len = Number(lenBigInt);
            const bytes = new Uint8Array(memory.buffer, ptr, len);
            const text = new TextDecoder("utf-8").decode(bytes);
            console.log(`[AutoZig] ${text}`);
        },
        
        js_error: (ptrBigInt, lenBigInt) => {
            const ptr = Number(ptrBigInt);
            const len = Number(lenBigInt);
            const bytes = new Uint8Array(memory.buffer, ptr, len);
            const text = new TextDecoder("utf-8").decode(bytes);
            console.error(`[AutoZig Error] ${text}`);
        }
    }
};

const { instance } = await WebAssembly.instantiateStreaming(
    fetch('your_module.wasm'),
    imports
);
```

## 📖 API Documentation

### Macros

#### `console_log!`

Output a log message to the browser console.

```rust
console_log!("Hello!");
console_log!("Count: {}", 42);
console_log!("Data: {:?}", my_vec);
```

#### `console_error!`

Output an error message to the browser console.

```rust
console_error!("Error occurred!");
console_error!("Code: {}", error_code);
```

### Functions

#### `init_panic_hook()`

Initialize panic hook to forward Rust panics to `console.error`.

```rust
init_panic_hook();
// Now all panics will be logged to the browser console
```

## 🏗️ Architecture

```text
┌─────────────────────────────────┐
│  Rust (User Code)               │
│  console_log!("Hello {}", name) │
└────────────┬────────────────────┘
             │ FFI call
             ↓
┌─────────────────────────────────┐
│  Zig (Middle Layer)             │
│  export fn autozig_log_impl()   │
└────────────┬────────────────────┘
             │ extern "env"
             ↓
┌─────────────────────────────────┐
│  JavaScript (Browser)           │
│  js_log(ptr, len)               │
│  console.log(text)              │
└─────────────────────────────────┘
```

## 🎯 Why AutoZig Console?

| Feature | `print!` | `wasm-bindgen` | **AutoZig Console** |
|:--------|:--------:|:--------------:|:-------------------:|
| Works in WASM | ❌ | ✅ | ✅ |
| WASM64 Support | ❌ | ⚠️ Limited | ✅ Native |
| Zero-copy | N/A | ❌ | ✅ |
| Type Safe | N/A | ⚠️ | ✅ |
| No Heavy Dependencies | N/A | ❌ | ✅ |
| User-friendly API | ✅ | ⚠️ | ✅ |

## 🔧 Build for WASM

### WASM32

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### WASM64

```bash
rustup install nightly
rustup target add wasm64-unknown-unknown --toolchain nightly
cargo +nightly build \
    --target wasm64-unknown-unknown \
    -Z build-std=std,panic_abort \
    --release
```

## 📚 Examples

See the [wasm64print example](https://github.com/layola13/autozig/tree/main/examples/wasm64print) for a complete working demonstration.

## 🤝 Integration with Other Crates

You can use `autozig-console` with any AutoZig-based WASM project:

```rust
// In your Cargo.toml
[dependencies]
autozig = "0.1"
autozig-console = "0.1"

// In your code
use autozig::autozig_export;
use autozig_console::{console_log, init_panic_hook};

#[autozig_export]
pub fn my_function(value: i32) -> i32 {
    console_log!("Processing value: {}", value);
    value * 2
}
```

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- Built on [AutoZig](https://github.com/layola13/autozig)
- Inspired by the need for better WASM logging solutions
- Thanks to the Rust and Zig communities

---

**Made with ❤️ for the Rust and Zig communities**