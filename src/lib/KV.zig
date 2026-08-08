//! A key-value pair.
const Self = @This();

const std = @import("std");
const Io = std.Io;

key: []const u8,
value: []const u8,

/// Iterates through a reader of key-value pairs.
pub const KVIterator = struct {
    reader: *Io.Reader,

    /// Go to the next key-value pair in the iterator.
    pub fn next(self: *KVIterator) ?Self {
        const key = self.reader.takeDelimiter('=') catch {
            return null;
        } orelse return null;
        const raw_value = self.reader.takeDelimiter('\n') catch {
            return null;
        } orelse return null;
        const value = std.mem.trimEnd(u8, raw_value, "\r");
        return .{
            .key = key,
            .value = value,
        };
    }
};

/// Createss an iterator used to parse key-value pairs.
pub fn parse(reader: *Io.Reader) KVIterator {
    return .{
        .reader = reader,
    };
}

const tgpa = std.testing.allocator;

fn attemptTest(buf: []const u8, kvs: []const ?Self) !void {
    var reader: Io.Reader = .fixed(buf);
    var parser = parse(&reader);
    var index: usize = 0;
    while (parser.next()) |kv| {
        try std.testing.expectEqualDeep(kvs[index].?, kv);
        index += 1;
    }
}

test "KV.start: success" {
    try attemptTest(
        "display_name=worker\r\ntags=exec::trap\r\nactive=true",
        &.{ .{ .key = "display_name", .value = "worker" }, .{ .key = "tags", .value = "exec::trap" }, .{ .key = "active", .value = "true" } },
    );
}

test "KV.start: only-key" {
    try attemptTest("display_name", &.{null});
}

test "KV.start: valueless" {
    try attemptTest("display_name=", &.{null});
}
