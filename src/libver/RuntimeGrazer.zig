const std = @import("std");
const builtin = @import("builtin");
const Io = std.Io;

/// The main filesystem layer to runtimes.
///
/// This is responsible for all filesystem interactions for runtimes.
const Self = @This();

const KV = @import("KV.zig");

unique_name: []const u8,
metadata: Metadata,
io: Io,
dir: Io.Dir,
home_path: []const u8,
environ_file_err: ?Io.File.OpenError = null,

pub const OpenError = Io.Dir.OpenError || Io.File.OpenError;
pub const EnvironError = std.mem.Allocator.Error || error{NoValue};

pub const Metadata = struct {
    display_name: []const u8,
};

pub fn open(io: Io, home: Io.Dir, home_path: []const u8, name: []const u8) OpenError!Self {
    const dir = try home.openDir(io, name, .{});
    const metadata_file = try dir.openFile(io, "metadata", .{});
    var buf: [255]u8 = undefined;
    var file_reader = metadata_file.reader(io, &buf);
    const reader = &file_reader.interface;

    var metadata: Metadata = .{
        .display_name = "",
    };
    var parser = KV.parse(reader);
    while (parser.next()) |kv| {
        if (std.mem.eql(u8, kv.key, "display_name")) {
            metadata.display_name = kv.value;
        }
    }

    return .{
        .unique_name = name,
        .metadata = metadata,
        .io = io,
        .dir = dir,
        .home_path = home_path,
    };
}

pub fn version(self: Self, version_num: []const u8) Io.Dir.OpenError!Io.Dir {
    return self.dir.openDir(self.io, version_num, .{});
}

/// Modifies `map` in place with the contents of a runtime's `environ` file.
pub fn environ(self: *Self, version_num: []const u8, allocator: std.mem.Allocator, map: *std.process.Environ.Map) EnvironError!void {
    const environ_file = self.dir.openFile(self.io, "environ", .{}) catch |err| {
        self.environ_file_err = err;
        return;
    };
    var buf: [255]u8 = undefined;
    var environ_reader = environ_file.reader(self.io, &buf);

    var parser = KV.parse(&environ_reader.interface);
    while (parser.next()) |kv| {
        const path_d = try std.mem.replaceOwned(u8, allocator, kv.value, "::", if (builtin.os.tag == .windows) ";" else ":");
        defer allocator.free(path_d);
        const version_d = try std.mem.replaceOwned(u8, allocator, path_d, "${version}", version_num);
        defer allocator.free(version_d);
        var sentinel = try std.mem.replaceOwned(u8, allocator, version_d, "${home}", self.home_path);
        defer allocator.free(sentinel);

        for (map.keys()) |key| {
            var needle: std.ArrayList(u8) = .empty;
            defer needle.deinit(allocator);
            try needle.appendSlice(allocator, "${");
            try needle.appendSlice(allocator, key);
            try needle.appendSlice(allocator, "}");
            const wack = try std.mem.replaceOwned(u8, allocator, sentinel, needle.items, map.get(key) orelse return EnvironError.NoValue);
            allocator.free(sentinel);
            sentinel = wack;
        }

        try map.put(kv.key, sentinel);
    }
}

pub fn deinit(self: *Self) void {
    self.dir.close(self.io);
}

const tio = std.testing.io;
const talloc = std.testing.allocator;

test "RuntimeGrazer.open: success" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();
    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    rt.deinit();
}

test "RuntimeGrazer.open: homeless" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    try std.testing.expectError(Io.Dir.OpenError.FileNotFound, open(tio, tdir, &tdir_root.sub_path, "runtime"));
}

test "RuntimeGrazer.open: metadataless" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    try tdir.createDirPath(tio, "runtime");
    try std.testing.expectError(Io.File.OpenError.FileNotFound, open(tio, tdir, &tdir_root.sub_path, "runtime"));
}

test "RuntimeGrazer.version: success" {
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
    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit();

    const vr = try rt.version("0.1.0");
    vr.close(tio);
}

test "RuntimeGrazer.version: fail" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();
    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit();

    try std.testing.expectError(Io.Dir.OpenError.FileNotFound, rt.version("0.1.0"));
}

test "RuntimeGrazer.environ: success" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();

    const env = try rtdir.createFile(tio, "environ", .{});
    var env_writer = env.writer(tio, &metadata_buf);
    try env_writer.interface.writeAll("PATH=bin::${PATH}");
    try env_writer.flush();

    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit();

    var map = try std.testing.environ.createMap(talloc);
    defer map.deinit();
    try rt.environ("0.1.0", talloc, &map);
}

test "RuntimeGrazer.environ: no environ" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();

    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit();

    var map = try std.testing.environ.createMap(talloc);
    defer map.deinit();
    try rt.environ("0.1.0", talloc, &map);
    try std.testing.expect(Io.File.OpenError.FileNotFound == rt.environ_file_err.?);
}

test "RuntimeGrazer.environ: allocator fail" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});
    defer rtdir.close(tio);

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();

    const env = try rtdir.createFile(tio, "environ", .{});
    var env_writer = env.writer(tio, &metadata_buf);
    try env_writer.interface.writeAll("PATH=bin:${PATH}");
    try env_writer.flush();

    var rt = try open(tio, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit();

    var map = try std.testing.environ.createMap(talloc);
    defer map.deinit();
    try std.testing.expectError(EnvironError.OutOfMemory, rt.environ("0.1.0", std.testing.failing_allocator, &map));
}
