//! By convention, root.zig is the root source file when making a package.
const std = @import("std");
const Io = std.Io;

pub const KV = @import("libver/KV.zig");

pub const RuntimeBuilder = struct {
    display_name: []const u8,
    search_paths: []const []const u8,
};

pub const RuntimeGrazer = struct {
    unique_name: []const u8,
    builder: RuntimeBuilder,
};

test {
    std.testing.refAllDecls(@This());
}
