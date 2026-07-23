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

pub const OpenError = Io.Dir.OpenError || Io.File.OpenError || std.mem.Allocator.Error;

pub const Metadata = struct {
    display_name: []const u8,
    search_paths: ?[]const []const u8,
};

pub fn open(io: Io, allocator: std.mem.Allocator, home: Io.Dir, name: []const u8) OpenError!Self {
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
    };
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
    const tdir = std.testing.tmpDir(.{}).dir;
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    var rt = try open(tio, talloc, tdir, "runtime");
    rt.deinit(talloc);
}

test "RuntimeGrazer.open: homeless" {
    const tdir = std.testing.tmpDir(.{}).dir;
    try std.testing.expectError(Io.Dir.OpenError.FileNotFound, open(tio, talloc, tdir, "runtime"));
}

test "RuntimeGrazer.open: metadataless" {
    const tdir = std.testing.tmpDir(.{}).dir;
    try tdir.createDirPath(tio, "runtime");
    try std.testing.expectError(Io.File.OpenError.FileNotFound, open(tio, talloc, tdir, "runtime"));
}

test "RuntimeGrazer.open: allocator error" {
    const tdir = std.testing.tmpDir(.{}).dir;
    const rtdir = try tdir.createDirPathOpen(tio, "runtime", .{});

    const metadata = try rtdir.createFile(tio, "metadata", .{});
    var metadata_buf: [16]u8 = undefined;
    var metadata_writer = metadata.writer(tio, &metadata_buf);
    try metadata_writer.interface.writeAll("display_name=Generic\nsearch_paths=abc");
    try std.testing.expectError(std.mem.Allocator.Error.OutOfMemory, open(tio, std.testing.failing_allocator, tdir, "runtime"));
}
