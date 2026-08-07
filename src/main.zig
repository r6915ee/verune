const std = @import("std");
const Io = std.Io;

const libver = @import("libver");
const clap = @import("clap");
const meta = @import("meta");

// These are our subcommands.
const SubCommands = enum {
    scope,
};

const main_parsers = .{
    .COMMAND = clap.parsers.enumeration(SubCommands),
    .ARGV = clap.parsers.string,
};

const main_params = clap.parseParamsComptime(
    \\-h, --help                           Display this help and exit.
    \\-v, --version                        Display the version and exit.
    \\<COMMAND>                            The subcommand to pass.
    \\
);

fn generateHelp(io: Io, params: []const clap.Param(clap.Help)) !void {
    const stderr_handle: Io.File = .stderr();
    var stderr_buffer: [255]u8 = undefined;
    var stderr = stderr_handle.writer(io, &stderr_buffer);

    try stderr.interface.print("usage: {s} ", .{@tagName(meta.name)});
    try stderr.flush();
    try clap.usage(&stderr.interface, clap.Help, params);
    try stderr.interface.writeAll("\n");
    try clap.help(&stderr.interface, clap.Help, params, .{ .description_indent = 4, .spacing_between_parameters = 0 });
    try stderr.flush();
}

pub fn main(init: std.process.Init) !void {
    var iter = try init.minimal.args.iterateAllocator(init.gpa);
    defer iter.deinit();

    _ = iter.next();

    var diag = clap.Diagnostic{};
    var res = clap.parseEx(clap.Help, &main_params, main_parsers, &iter, .{
        .diagnostic = &diag,
        .allocator = init.gpa,
        .terminating_positional = 0,
    }) catch |err| {
        try diag.reportToFile(init.io, .stderr(), err);
        return err;
    };
    defer res.deinit();

    if (res.args.version != 0) {
        const stdout_handle: Io.File = .stdout();
        var stdout_buffer: [6]u8 = undefined;
        var stdout = stdout_handle.writer(init.io, &stdout_buffer);

        try stdout.interface.print("{s}", .{meta.version});
        return stdout.flush();
    }
    if (res.args.help != 0)
        return generateHelp(init.io, &main_params);

    const command = res.positionals[0] orelse return error.MissingCommand;
    switch (command) {
        .scope => try scopeMain(init.io, init.gpa, &iter),
    }
}

fn scopeMain(io: Io, gpa: std.mem.Allocator, iter: *std.process.Args.Iterator) !void {
    const params = comptime clap.parseParamsComptime(
        \\-h, --help  Display this help and exit.
        \\<ARGV>...   A list of arguments to run in the scope; the first argument is the command.
        \\
    );

    var diag = clap.Diagnostic{};
    var res = clap.parseEx(clap.Help, &params, main_parsers, iter, .{
        .diagnostic = &diag,
        .allocator = gpa,
    }) catch |err| {
        try diag.reportToFile(io, .stderr(), err);
        return err;
    };
    defer res.deinit();

    if (res.args.help != 0)
        return generateHelp(io, &params);
}
