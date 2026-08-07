//! By convention, root.zig is the root source file when making a package.
const std = @import("std");
const Io = std.Io;

/// A key-value pair.
pub const KV = @import("libver/KV.zig");
/// The main filesystem layer to runtimes.
///
/// `RuntimeGrazer` is responsible for every IO operation with runtimes.
/// One `RuntimeGrazer` handles one single runtime.
pub const RuntimeGrazer = @import("libver/RuntimeGrazer.zig");
/// Manages scopes and interacting with projects.
pub const ScopeSentinel = @import("libver/ScopeSentinel.zig");

test {
    std.testing.refAllDecls(@This());
}
