const std = @import("std");

// String operations module
// Demonstrates that each module can have its own allocator usage without conflicts

export fn string_length(ptr: [*:0]const u8) usize {
    var len: usize = 0;
    while (ptr[len] != 0) : (len += 1) {}
    return len;
}

export fn string_compare(a: [*:0]const u8, b: [*:0]const u8) i32 {
    var i: usize = 0;
    while (true) {
        if (a[i] == 0 and b[i] == 0) return 0;
        if (a[i] == 0) return -1;
        if (b[i] == 0) return 1;
        if (a[i] < b[i]) return -1;
        if (a[i] > b[i]) return 1;
        i += 1;
    }
}

export fn string_hash(ptr: [*:0]const u8) u64 {
    var hash: u64 = 5381;
    var i: usize = 0;
    while (ptr[i] != 0) : (i += 1) {
        hash = ((hash << 5) +% hash) +% ptr[i];
    }
    return hash;
}
