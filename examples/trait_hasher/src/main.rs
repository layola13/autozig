//! Trait Hasher Example - Phase 2: Stateful Traits with Opaque Pointers
//!
//! This example demonstrates AutoZig's Phase 2 stateful trait support.
//! It shows how Zig code can implement Rust's std::hash::Hasher trait
//! with automatic lifecycle management through opaque pointers.

use autozig::autozig;
use std::collections::HashMap;
use std::hash::{Hash, BuildHasherDefault, Hasher};

// AutoZig macro: Zig implementation + Rust trait mapping with opaque pointer
autozig! {
    // Zig side: Stateful hasher implementation
    const std = @import("std");

    const HasherState = struct {
        sum: u64,
        written: usize,
    };

    export fn hasher_new() ?*HasherState {
        const ptr = std.heap.c_allocator.create(HasherState) catch return null;
        ptr.* = .{ .sum = 0, .written = 0 };
        return ptr;
    }

    export fn hasher_free(ptr: *HasherState) void {
        std.heap.c_allocator.destroy(ptr);
    }

    export fn hasher_write(self: *HasherState, bytes_ptr: [*]const u8, len: usize) void {
        const bytes = bytes_ptr[0..len];
        for (bytes) |byte| {
            self.sum = self.sum *% 31 +% @as(u64, byte);
        }
        self.written += len;
    }

    export fn hasher_finish(self: *const HasherState) u64 {
        return self.sum;
    }

    ---

    // Rust side: Opaque pointer wrapper with lifecycle management
    struct ZigHasher(opaque);

    impl ZigHasher {
        #[constructor]
        fn new() -> Self {
            hasher_new()
        }

        #[destructor]
        fn drop(&mut self) {
            hasher_free()
        }
    }

    impl std::hash::Hasher for ZigHasher {
        fn write(&mut self, bytes: &[u8]) {
            hasher_write(bytes)
        }
        
        fn finish(&self) -> u64 {
            hasher_finish()
        }
    }
}

fn main() {
    println!("=== AutoZig Phase 2: Stateful Trait Hasher ===\n");

    // Demo 1: Basic hasher usage
    println!("Demo 1: Basic Hashing");
    println!("---------------------");
    let mut hasher = ZigHasher::new();
    hasher.write(b"hello");
    let hash = hasher.finish();
    println!("Hash of 'hello': {}", hash);
    println!();

    // Demo 2: Hash consistency
    println!("Demo 2: Hash Consistency");
    println!("------------------------");
    let mut h1 = ZigHasher::new();
    h1.write(b"test");
    let hash1 = h1.finish();

    let mut h2 = ZigHasher::new();
    h2.write(b"test");
    let hash2 = h2.finish();

    println!("Hash 1: {}", hash1);
    println!("Hash 2: {}", hash2);
    println!("Consistent: {}", hash1 == hash2);
    println!();

    // Demo 3: Using with HashMap
    println!("Demo 3: HashMap Integration");
    println!("----------------------------");
    
    type ZigHashMap<K, V> = HashMap<K, V, BuildHasherDefault<ZigHasher>>;
    let mut map: ZigHashMap<String, String> = HashMap::default();
    
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());
    
    println!("map['key1'] = {:?}", map.get("key1"));
    println!("map['key2'] = {:?}", map.get("key2"));
    println!("map size: {}", map.len());
    println!();

    // Demo 4: Multiple writes
    println!("Demo 4: Multiple Writes");
    println!("-----------------------");
    let mut hasher = ZigHasher::new();
    hasher.write(b"hello");
    hasher.write(b" ");
    hasher.write(b"world");
    println!("Hash of 'hello world': {}", hasher.finish());
    println!();

    // Demo 5: Lifecycle management (Drop)
    println!("Demo 5: Automatic Cleanup");
    println!("-------------------------");
    {
        let _hasher = ZigHasher::new();
        println!("Hasher created in inner scope");
    } // Drop called here automatically
    println!("Hasher automatically freed (Drop called)");
    println!();

    // Demo 6: Zero-length input
    println!("Demo 6: Edge Cases");
    println!("------------------");
    let mut hasher = ZigHasher::new();
    hasher.write(b"");
    println!("Hash of empty string: {}", hasher.finish());
    println!();

    println!("=== All demos completed successfully! ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hashing() {
        let mut hasher = ZigHasher::new();
        hasher.write(b"test");
        let hash = hasher.finish();
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_hash_consistency() {
        let mut h1 = ZigHasher::new();
        h1.write(b"test");
        let hash1 = h1.finish();

        let mut h2 = ZigHasher::new();
        h2.write(b"test");
        let hash2 = h2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hashmap_integration() {
        type ZigHashMap<K, V> = HashMap<K, V, BuildHasherDefault<ZigHasher>>;
        let mut map: ZigHashMap<String, i32> = HashMap::default();
        
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);

        assert_eq!(map.get("one"), Some(&1));
        assert_eq!(map.get("two"), Some(&2));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_drop() {
        // Test that Drop is correctly called
        // This test passes if no memory leak occurs (verify with valgrind)
        let _hasher = ZigHasher::new();
        // Drop is called automatically at end of scope
    }

    #[test]
    fn test_multiple_writes() {
        let mut hasher = ZigHasher::new();
        hasher.write(b"hello");
        hasher.write(b" ");
        hasher.write(b"world");
        let hash = hasher.finish();
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_zero_length() {
        let mut hasher = ZigHasher::new();
        hasher.write(b"");
        let hash = hasher.finish();
        assert_eq!(hash, 0); // Empty string should hash to 0
    }

    #[test]
    fn test_different_inputs_different_hashes() {
        let mut h1 = ZigHasher::new();
        h1.write(b"hello");
        let hash1 = h1.finish();

        let mut h2 = ZigHasher::new();
        h2.write(b"world");
        let hash2 = h2.finish();

        assert_ne!(hash1, hash2);
    }
}