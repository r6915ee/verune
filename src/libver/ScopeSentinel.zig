const std = @import("std");
const builtin = @import("builtin");
const Io = std.Io;

/// Manages environments for scopes.
const Self = @This();

const KV = @import("KV.zig");
const RuntimeGrazer = @import("RuntimeGrazer.zig");

conf: ConfFormat,

pub const ConfFormat = std.StringHashMap([]const u8);
pub const EnvironError = Io.Dir.RealPathError || RuntimeGrazer.OpenError || std.process.Environ.CreateMapError || RuntimeGrazer.EnvironError;

/// Initialize a `ScopeSentinel`, using a [`Reader`](#std.Io.Reader) that goes through the `.version` file.
/// Remember to run `deinit` once you're done!
pub fn open(reader: *Io.Reader, allocator: std.mem.Allocator) std.mem.Allocator.Error!Self {
    var conf: ConfFormat = .init(allocator);
    errdefer conf.deinit();
    var parser = KV.parse(reader);
    while (parser.next()) |kv| {
        try conf.put(kv.key, kv.value);
    }

    return .{
        .conf = conf,
    };
}

/// Go through every runtime in the configuration and load its environment variables into `map`.
///
/// Note that any changes to `map` are destructive, as this function won't restore `map` to its
/// previous state when an error is propagated.
pub fn environ(self: Self, io: Io, allocator: std.mem.Allocator, home: Io.Dir, home_path: []const u8, map: *std.process.Environ.Map) EnvironError!void {
    var conf_iter = self.conf.iterator();
    while (conf_iter.next()) |conf_item| {
        var grazer = try RuntimeGrazer.open(io, home, home_path, conf_item.key_ptr.*);
        defer grazer.deinit();
        try grazer.environ(conf_item.value_ptr.*, allocator, map);
    }
}

/// Deinitialize the configuration hashmap.
pub fn deinit(self: *Self) void {
    self.conf.deinit();
}

const tio = std.testing.io;
const talloc = std.testing.allocator;

test "ScopeSentinel.open: success" {
    const buf: []const u8 = "runtime=0.1.0";
    var reader: Io.Reader = .fixed(buf);

    var sentinel = try open(&reader, talloc);
    defer sentinel.deinit();
}

test "ScopeSentinel.open: allocator fail" {
    const buf: []const u8 = "runtime=0.1.0";
    var reader: Io.Reader = .fixed(buf);

    try std.testing.expectError(std.mem.Allocator.Error.OutOfMemory, open(&reader, std.testing.failing_allocator));
}

test "ScopeSentinel.exec: success" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);
    _ = try rtdir.createDirPath(tio, "0.1.0");

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();

    const env = try rtdir.createFile(tio, "environ", .{});
    var env_writer = env.writer(tio, &metadata_buf);
    try env_writer.interface.writeAll("PATH=bin::${PATH}");
    try env_writer.flush();

    const buf: []const u8 = "runtime=0.1.0";
    var reader: Io.Reader = .fixed(buf);

    var sentinel = try open(&reader, talloc);
    defer sentinel.deinit();

    var map = try std.testing.environ.createMap(talloc);
    defer map.deinit();
    try sentinel.environ(tio, talloc, tdir, &tdir_root.sub_path, &map);
}

test "ScopeSentinel.exec: rt error" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);

    const buf: []const u8 = "runtime=0.1.0";
    var reader: Io.Reader = .fixed(buf);

    var sentinel = try open(&reader, talloc);
    defer sentinel.deinit();

    var map = try std.testing.environ.createMap(talloc);
    defer map.deinit();
    try std.testing.expectError(EnvironError.FileNotFound, sentinel.environ(tio, talloc, tdir, &tdir_root.sub_path, &map));
}
