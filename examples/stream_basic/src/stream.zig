const std = @import("std");

// Stream callback type definitions
pub const StreamDataCallback = fn (data: [*]const u8, len: usize) callconv(.C) void;
pub const StreamErrorCallback = fn (error_msg: [*:0]const u8) callconv(.C) void;
pub const StreamCompleteCallback = fn () callconv(.C) void;

// Stream handle for managing stream state
pub const StreamHandle = struct {
    on_data: ?StreamDataCallback,
    on_error: ?StreamErrorCallback,
    on_complete: ?StreamCompleteCallback,
    is_active: bool,

    pub fn init() StreamHandle {
        return StreamHandle{
            .on_data = null,
            .on_error = null,
            .on_complete = null,
            .is_active = false,
        };
    }

    pub fn setDataCallback(self: *StreamHandle, callback: StreamDataCallback) void {
        self.on_data = callback;
    }

    pub fn setErrorCallback(self: *StreamHandle, callback: StreamErrorCallback) void {
        self.on_error = callback;
    }

    pub fn setCompleteCallback(self: *StreamHandle, callback: StreamCompleteCallback) void {
        self.on_complete = callback;
    }

    pub fn start(self: *StreamHandle) void {
        self.is_active = true;
    }

    pub fn stop(self: *StreamHandle) void {
        self.is_active = false;
    }

    pub fn sendData(self: *StreamHandle, data: []const u8) void {
        if (self.is_active and self.on_data != null) {
            self.on_data.?(data.ptr, data.len);
        }
    }

    pub fn sendError(self: *StreamHandle, error_msg: [*:0]const u8) void {
        if (self.on_error != null) {
            self.on_error.?(error_msg);
        }
    }

    pub fn complete(self: *StreamHandle) void {
        self.is_active = false;
        if (self.on_complete != null) {
            self.on_complete.?();
        }
    }
};

// Example: Counter stream that produces numbers from 0 to max-1
export fn zig_create_counter_stream(max: u32, callback: StreamDataCallback) void {
    var handle = StreamHandle.init();
    handle.setDataCallback(callback);
    handle.start();

    var i: u32 = 0;
    while (i < max) : (i += 1) {
        // Convert u32 to bytes (little-endian)
        const bytes = std.mem.asBytes(&i);
        handle.sendData(bytes);
    }

    handle.complete();
}

// Example: Stream that produces fibonacci numbers
export fn zig_fibonacci_stream(count: u32, callback: StreamDataCallback) void {
    var handle = StreamHandle.init();
    handle.setDataCallback(callback);
    handle.start();

    var a: u32 = 0;
    var b: u32 = 1;
    var i: u32 = 0;

    while (i < count) : (i += 1) {
        const bytes = std.mem.asBytes(&a);
        handle.sendData(bytes);

        const temp = a + b;
        a = b;
        b = temp;
    }

    handle.complete();
}

// Test functions for verification
test "StreamHandle init" {
    const handle = StreamHandle.init();
    try std.testing.expect(handle.on_data == null);
    try std.testing.expect(handle.on_error == null);
    try std.testing.expect(handle.on_complete == null);
    try std.testing.expect(handle.is_active == false);
}

test "StreamHandle callbacks" {
    var handle = StreamHandle.init();

    const testCallback = struct {
        fn dataCallback(data: [*]const u8, len: usize) callconv(.C) void {
            _ = data;
            _ = len;
        }
    }.dataCallback;

    handle.setDataCallback(testCallback);
    try std.testing.expect(handle.on_data != null);
}
