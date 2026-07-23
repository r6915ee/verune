const std = @import("std");
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

pub const OpenError = Io.Dir.OpenError || Io.File.OpenError || std.mem.Allocator.Error;

pub const Metadata = struct {
    display_name: []const u8,
    search_paths: ?[]const []const u8,
};

pub fn open(io: Io, allocator: std.mem.Allocator, home: Io.Dir, home_path: []const u8, name: []const u8) OpenError!Self {
    const dir = try home.openDir(io, name, .{});
    const metadata_file = try dir.openFile(io, "metadata", .{});
    var buf: [255]u8 = undefined;
    var file_reader = metadata_file.reader(io, &buf);
    const reader = &file_reader.interface;

    var metadata: Metadata = .{
        .display_name = "",
        .search_paths = &.{},
    };
    var parser = KV.parse(reader);
    while (parser.next()) |kv| {
        if (std.mem.eql(u8, kv.key, "display_name")) {
            metadata.display_name = kv.value;
        } else if (std.mem.eql(u8, kv.key, "search_paths")) {
            if (kv.value.len == 0) {
                continue;
            }
            if (metadata.search_paths) |search_paths| {
                allocator.free(search_paths);
                metadata.search_paths = null;
            }
            var paths: std.ArrayList([]const u8) = .empty;
            errdefer paths.deinit(allocator);
            var iter = std.mem.splitSequence(u8, kv.value, "::");
            while (iter.next()) |path| {
                try paths.append(allocator, path);
            }
            metadata.search_paths = try paths.toOwnedSlice(allocator);
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

pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
    self.dir.close(self.io);
    if (self.metadata.search_paths) |search_paths| {
        allocator.free(search_paths);
    }
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
    var rt = try open(tio, talloc, tdir, &tdir_root.sub_path, "runtime");
    rt.deinit(talloc);
}

test "RuntimeGrazer.open: homeless" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    try std.testing.expectError(Io.Dir.OpenError.FileNotFound, open(tio, talloc, tdir, &tdir_root.sub_path, "runtime"));
}

test "RuntimeGrazer.open: metadataless" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    try tdir.createDirPath(tio, "runtime");
    try std.testing.expectError(Io.File.OpenError.FileNotFound, open(tio, talloc, tdir, &tdir_root.sub_path, "runtime"));
}

test "RuntimeGrazer.open: allocator error" {
    const tdir_root = std.testing.tmpDir(.{});
    const tdir = tdir_root.dir;
    defer tdir.close(tio);
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try metadata_writer.flush();
    try std.testing.expectError(std.mem.Allocator.Error.OutOfMemory, open(tio, std.testing.failing_allocator, tdir, &tdir_root.sub_path, "runtime"));
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
    var rt = try open(tio, talloc, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit(talloc);

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
    var rt = try open(tio, talloc, tdir, &tdir_root.sub_path, "runtime");
    defer rt.deinit(talloc);

    try std.testing.expectError(Io.Dir.OpenError.FileNotFound, rt.version("0.1.0"));
}
