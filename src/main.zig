const std = @import("std");
const builtin = @import("builtin");
const Io = std.Io;

const libver = @import("libver");
const clap = @import("clap");
const meta = @import("meta");

// These are our subcommands.
const SubCommands = enum {
    scope,
    filter,
};

const main_parsers = .{
    .COMMAND = clap.parsers.enumeration(SubCommands),
    .ARGV = clap.parsers.string,
    .PATH = clap.parsers.string,
    .@"RUNTIME=VERSION" = clap.parsers.string,
};

const main_params = clap.parseParamsComptime(
    \\-h, --help                           Display this help and exit.
    \\-v, --version                        Display the version and exit.
    \\-r, --replace <RUNTIME=VERSION>...   Overlay a single runtime version.
    \\-o, --overlay <PATH>...              Overlay a different configuration.
    \\<COMMAND>                            The subcommand to pass.
    \\
);

const MainArgs = clap.ResultEx(clap.Help, &main_params, main_parsers);

fn generateHelp(io: Io, version: ?[]const u8, params: []const clap.Param(clap.Help)) !void {
    const stderr_handle: Io.File = .stderr();
    var stderr_buffer: [255]u8 = undefined;
    var stderr = stderr_handle.writer(io, &stderr_buffer);

    try stderr.interface.print("usage: {s} ", .{@tagName(meta.name)});
    try stderr.flush();
    if (version) |v| {
        try stderr.interface.print("{s} ", .{v});
        try stderr.flush();
    }
    try clap.usage(&stderr.interface, clap.Help, params);
    try stderr.interface.writeAll("\n");
    try clap.help(&stderr.interface, clap.Help, params, .{ .description_indent = 4, .spacing_between_parameters = 0 });
    try stderr.flush();
}

fn generateScope(io: Io, gpa: std.mem.Allocator, dir: Io.Dir, map: *std.process.Environ.Map, main_args: MainArgs, conf_buf: []u8) !libver.ScopeSentinel {
    const conf_file = try dir.openFile(io, map.get("VERUNE_CONFIG") orelse ".version", .{});
    defer conf_file.close(io);

    var conf_buf_writer: Io.Writer = .fixed(conf_buf);

    // `conf_buf` must be allocated separately from this function, because `ScopeSentinel` doesn't (and shouldn't) own the memory to the key-value pairs.
    // I think the stack memory address gets naturally reused by Io.Reader, so the keys and values are still the same memory.
    var conf_file_reader = conf_file.reader(io, conf_buf);
    _ = try conf_file_reader.interface.streamRemaining(&conf_buf_writer);

    if (map.get("VERUNE_OVERLAYS")) |overlays| {
        var iter = std.mem.splitScalar(u8, overlays, if (builtin.os.tag == .windows) ';' else ':');
        while (iter.next()) |x| {
            const x_file = try dir.openFile(io, x, .{});
            defer x_file.close(io);

            var x_file_reader = x_file.reader(io, conf_buf);
            _ = try x_file_reader.interface.streamRemaining(&conf_buf_writer);
        }
    }

    for (main_args.args.overlay) |x| {
        const x_file = try dir.openFile(io, x, .{});
        defer x_file.close(io);

        var x_file_reader = x_file.reader(io, conf_buf);
        _ = try x_file_reader.interface.streamRemaining(&conf_buf_writer);
    }

    var scope: libver.ScopeSentinel = .new(gpa);
    errdefer scope.deinit();
    var conf_buf_reader: Io.Reader = .fixed(conf_buf);
    try scope.interp(&conf_buf_reader);

    for (main_args.args.replace) |x| {
        var reader: Io.Reader = .fixed(x);
        var parser = libver.KV.parse(&reader);
        while (parser.next()) |kv| {
            try scope.conf.put(kv.key, kv.value);
        }
    }

    return scope;
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
        return generateHelp(init.io, null, &main_params);

    const map = init.environ_map;

    const home_path: []const u8 = map.get("VERUNE_HOME") orelse (if (builtin.os.tag == .windows)
        try std.fmt.allocPrint(init.gpa, "{s}\\.ver", .{map.get("HOMEPATH").?})
    else
        try std.fmt.allocPrint(init.gpa, "{s}/.ver", .{map.get("HOME").?}));
    defer init.gpa.free(home_path);

    const command = res.positionals[0] orelse return error.MissingCommand;
    switch (command) {
        .scope => try scopeMain(init.io, init.gpa, .cwd(), map, &iter, home_path, res),
        .filter => try filterMain(init.io, init.gpa, .cwd(), map, &iter, res),
    }
}

fn scopeMain(io: Io, gpa: std.mem.Allocator, dir: Io.Dir, map: *std.process.Environ.Map, iter: *std.process.Args.Iterator, home_path: []const u8, main_args: MainArgs) !void {
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
        return generateHelp(io, "scope", &params);

    var conf_buf: [255]u8 = undefined;
    var scope = try generateScope(io, gpa, dir, map, main_args, &conf_buf);
    defer scope.deinit();

    const home: Io.Dir = try dir.openDir(io, home_path, .{});
    defer home.close(io);

    try scope.environ(io, gpa, home, home_path, map);

    const scope_level = try std.fmt.parseInt(u8, map.get("VERUNE_SCOPE_LEVEL") orelse "0", 10);
    var scope_level_buf: [3]u8 = undefined;
    try map.put("VERUNE_SCOPE_LEVEL", try std.fmt.bufPrint(&scope_level_buf, "{d}", .{scope_level + 1}));

    const argv: []const []const u8 = if (res.positionals[0].len > 0) res.positionals[0] else if (map.get("SHELL")) |sh| &.{sh} else if (builtin.os.tag == .windows) &.{"cmd"} else &.{"sh"};

    return std.process.replace(io, .{ .argv = argv, .environ_map = map });
}

fn filterMain(io: Io, gpa: std.mem.Allocator, dir: Io.Dir, map: *std.process.Environ.Map, iter: *std.process.Args.Iterator, main_args: MainArgs) !void {
    const params = comptime clap.parseParamsComptime(
        \\-h, --help         Display this help and exit.
        \\-i, --installed    List only installed runtimes
        \\-u, --uninstalled  List only uninstalled runtimes
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
        return generateHelp(io, "filter", &params);

    var conf_buf: [255]u8 = undefined;
    var scope = try generateScope(io, gpa, dir, map, main_args, &conf_buf);
    defer scope.deinit();

    var i_balance: u2 = 1;
    if (res.args.installed != 0)
        i_balance += 1;
    if (res.args.uninstalled != 0)
        i_balance -= 1;

    var ite = scope.conf.iterator();
    while (ite.next()) |x| {
        const y = try std.fmt.allocPrint(gpa, "{s}{c}{s}", .{ x.key_ptr.*, if (builtin.os.tag == .windows) '\\' else '/', x.value_ptr.* });
        defer gpa.free(y);

        const z = if (dir.access(io, y, .{})) |_| true else |_| false;

        if ((i_balance == 0 and z == false) or (i_balance == 2 and z == true))
            continue;

        const stdout_handle: Io.File = .stdout();
        var stdout_buf: [64]u8 = undefined;
        var stdout = stdout_handle.writer(io, &stdout_buf);

        try stdout.interface.print("{s}={s}\n", .{ x.key_ptr.*, x.value_ptr.* });
        try stdout.flush();
    }
}
