# `verune`

> _Dead simple, generic runtime version manager_

Software development relies on the concept of runtimes, which are responsible
for invoking compilers, interpreters, virtual machines, and more. They make
developing in a programming language easier and faster, alongside providing
performance benefits.

Unfortunately, there isn't a defined way to manage them in a safe and portable
manner. For example, most package managers won't allow having multiple versions
installed on a system, which can cause problems with breaking changes.

`verune` aims to mitigate this by treating projects as what define runtime
runtime versions, helping both developers and users get up to speed and safe on
development.

The design of `verune` means it can be used with unusual forms of toolchains,
such as game engines.

## Installation

`verune` provides binaries on its [releases
page](https://codeberg.org/r6915ee/verune/); choose the one most suitable for
your platform.

### Building

You can also use the [Zig toolchain](https://ziglang.org/) to build `verune`.
Using version v0.16.0, you can run any of the build steps:

```sh
zig build # Build a binary.
zig build run # Build and run a binary.
zig build test # Run any unit tests.
zig build docs # Build the libver docs.
```

## Usage

### Basic

#### Runtimes

To get started using a runtime, you first need to set a runtime home, which
is where all of your runtimes and their metadata are stored. This is set to
the `~/.ver` directory by default, assuming `~` refers to the home directory.
You can change where the runtime home is located by using the `VERUNE_HOME`
environment variable.

Each runtime needs to have its own folder in the home. Subdirectories under
this folder will be treated as runtime versions For example, a valid Zig
installation would look like this:

```
└── zig
    ├── 0.16.0
    ├── environ
    └── metadata
```

The files that define a runtime the most are the `metadata` file and the
`environ` file. Both are written in a basic key-value pair language that is
similar to the INI format. `metadata` only has a single field, that being
`display_name`, which is primarily useful for different user interfaces:

```
display_name=Zig
```

The `environ` file is more interesting, however. It accepts any key-value
pairs, and each pair will be set as an environment variable:

```
PATH=${home}/zig/${version}::${PATH}
```

Values are encouraged to use substitution, which will replace substitution
tokens with something else. `${home}` will match to the current runtime home,
`${version}` will match to the currently selected runtime version, `::` will
match to the current system's preferred `PATH` delimiter, and anything else
enclosed in `${` and `}` will be treated as an environment variable. This file
is one of the more powerful features of `verune` due to its substitution and
environment modification, and every runtime is expected to have one in order
to provide the binaries of the runtime.

You can pretty much do anything with the `environ` file, such as providing
wrappers over a runtime, setting up libraries, and more. Some examples of its
capabilities include:

- Running a runtime in a sandbox
- Adding a cross-compilation wrapper
- Serving shared and static libraries

#### Projects

Finally, we need to add support for `verune` into a project. In a project's
repository, create a `.version` file and write the following string in it:

```
zig=0.16.0
```

That's it! Now you just need to enter a "scope", which will get your
environment ready for you:

```sh
verune scope
$ zig version # 0.16.0
$ exit
zig version # Since we exited out of the scope already, Zig won't be available.
```

#### Filter

The second subcommand available in `verune` is `filter`, which lists every
runtime version that the project is using. By default, it will print every
runtime version in use:

```sh
verune filter
# zig=0.16.0
```

However, you can also add various flags to `filter`. For example, `--installed`
will only list installed runtimes:

```sh
verune filter --installed
# zig=0.16.0
```

...and `--uninstalled` will only list uninstalled ones:

```sh
verune filter --uninstalled
```

Using `filter`, you can check which runtimes you need to install for a project
using `verune`.

### Advanced

#### Configuration

`verune` can use a different file for project configuration by using the
`VERUNE_CONFIG` environment variable.

```sh
echo zig=0.16.0 > conf
VERUNE_CONFIG=conf verune scope
$ zig version # 0.16.0
```

However, `verune` also has a concept called _overlays_. An overlay is created
by using at least one _overlay input_, which is an additional piece of
configuration that gets loaded.

The most basic input is the `--replace` option, which uses a string that is
very similar to a regular configuration file, and comes after every other
input:

```sh
verune --replace zig=0.16.0 scope
$ zig version # 0.16.0
```

The `--overlay` option will instead read from a dedicated file and comes in
the middle:

```sh
verune --overlay conf scope
$ zig version # 0.16.0
```

Finally, the `VERUNE_OVERLAYS` environment variable lists files to use as
inputs and comes before everything else:

```sh
echo haxe=4.3.7 > alt
VERUNE_OVERLAYS=conf:alt verune scope
$ zig version # 0.16.0
$ haxe --version # 4.3.7
```

#### Environment Variables

`verune` sets a special environment variable in scopes called
`VERUNE_SCOPE_LEVEL`. It's a number that notes how far down a scope currently
is, in the case that it may be nested:

```sh
verune scope
$ echo $VERUNE_SCOPE_LEVEL # 1
$ verune scope
$ echo $VERUNE_SCOPE_LEVEL # 2
$ exit
$ echo $VERUNE_SCOPE_LEVEL # 1
$ exit
echo $VERUNE_SCOPE_LEVEL
```

#### Dropping Into Programs

You can immediately drop into a different program than your shell by providing
at least one argument to `scope`. This even includes programs provided by a
runtime:

```sh
verune scope zig version # 0.16.0
```

## Tips and Tricks

- It is generally a very good idea to run development tools inside of a
  `verune` scope. This should be done in order to provide the pinned runtimes
  to said tools.
- It is **very** crucial to place any configuration files for `verune` in the
  version control system for a repository, as it can easily be shared by anyone
  interacting with the repository's code.
- Although it isn't a good idea, it is possible to replicate shims in other
  version managers by writing a script to drop into a runtime.
- Other programs can use `verune`'s capabilities by making use of its `libver`
  module, whose code lives in the same repository.

## License

`verune` and `libver` are dual-licensed under the **MIT** and **Apache 2.0**
licenses. Contributions must be licensed in this manner.
