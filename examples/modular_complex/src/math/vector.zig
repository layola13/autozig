const std = @import("std");

// Vector2D operations - demonstrates modular Zig code organization
// Use extern struct for FFI compatibility
pub const Vector2D = extern struct {
    x: f32,
    y: f32,

    pub fn init(x: f32, y: f32) Vector2D {
        return Vector2D{ .x = x, .y = y };
    }

    pub fn add(self: Vector2D, other: Vector2D) Vector2D {
        return Vector2D{
            .x = self.x + other.x,
            .y = self.y + other.y,
        };
    }

    pub fn length(self: Vector2D) f32 {
        return @sqrt(self.x * self.x + self.y * self.y);
    }

    pub fn dot(self: Vector2D, other: Vector2D) f32 {
        return self.x * other.x + self.y * other.y;
    }
};

// Export FFI functions
export fn vector_create(x: f32, y: f32) Vector2D {
    return Vector2D.init(x, y);
}

export fn vector_add(a: Vector2D, b: Vector2D) Vector2D {
    return a.add(b);
}

export fn vector_length(v: Vector2D) f32 {
    return v.length();
}

export fn vector_dot(a: Vector2D, b: Vector2D) f32 {
    return a.dot(b);
}
