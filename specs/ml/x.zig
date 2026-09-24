const base = @import("../base/t.zig");

test "relative import test" {
    // This should work when compiled from specs/ root directory
    _ = base;
}