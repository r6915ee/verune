# `verune`

> _Dead simple, generic runtime version manager_

Software development often involves the usage of runtimes, such as compilers or
interpreters, that allow building software systems. Often, these runtimes are
managed using system package managers.

The main drawback to the approach of installing runtimes as packages, however,
is that they are not pinned in the slightest. Imagine a runtime at some point
had a breaking change that impacted some software; if somebody attempted to use
that software with a version of the runtime that included this breaking change,
it would be impossible to safely use the software.

`verune` solves this issue on a project-based manner: a configuration gets
created by a developer that pinpoints what version of each runtime is needed,
and other developers and/or users can then use or develop that software using
only that version.

## Installation

`verune` can be installed from a global Cargo installation using `cargo
install`.

```sh
cargo install verune # Install from Crates.io
cargo install --git https://codeberg.org/r6915ee/verune.git # Install using Git
```

## Usage

### Runtimes

Runtimes must be installed in a portable fashion: that is, they cannot be
managed by another program, such as a system package manager or a Windows
installer.

Runtimes go under `~/.ver/`, where the tilde is equivalent to the home
directory. A simple way to look at this design would be a tree:

```
~/.ver/
|-haxe/
  |-meta.ron
|-bun/
  |-meta.ron
|-rust/
  |-meta.ron
```

Runtime metadata can be declared using `meta.ron` files, which are in the
[RON](https://github.com/ron-rs/ron/) format. The format in particular is
similar to _JSON_, but introduces some changes that make readability easier.

Assuming we've chosen a sample as our first runtime to install, the first step
is to create its associated directory under the runtime directory. Then, all we
need to do is run `verune`'s `template` subcommand:

```sh
verune template runtime
```

This will create a template metadata file for us under `~/.ver/runtime/meta.ron`.
This file contains the following:

```ron
(
    display_name: "",
    search_paths: [],
)
```

`display_name` is primarily useful for GUI programs. What is of particular
interest is the `search_paths` field, however. Any external programs can use
this field's data as paths to search for runtime-specific programs that are
hidden deeper in a version's installation; for example:

```
~/.ver/runtime/
|-1.0.0/
  |-prog
  |-dir/
    |-two/
      |-pkgman
```

If we wanted to be able to execute `pkgman` in this example, we can simply add
its relative parent directory into the `search_paths` field. Thus, a possible
example of a metadata file for this runtime would be:

```ron
(
    display_name: "Runtime",
    search_paths: ["dir/two"],
)
```

Now we can get into setting up projects.

### Projects

Assuming we use the prior runtime example with `prog` and `pkgman`, we know
that our project uses version v1.0.0 of that runtime. We can create a project
configuration using the `switch` subcommand:

```sh
verune switch runtime 1.0.0
```

Normally, this will create and write to `.ver.ron`, which is where all runtime
version information is stored using [RON](https://github.com/ron-rs/ron/). If
we specify a version that is not installed, however, we get an error. If you're
just looking to tell the program which version you want to use, you may do so
by using the `-u`/`--skip-check` flag.

```sh
verune switch runtime 1.0.1 # Error!
verune switch -u runtime 1.0.1 # Success
```

Of course, it's better to use the former approach to switching versions, as
it's simply better to be safe than sorry in the case of version management.

An interesting thing to note is that `.ver.ron` isn't the only possible
location for a configuration file. The configuration file location can be
controlled using the `-c`/`--config` flag and the `VER_CONFIG` environment
variable, which allow for using multiple configuration files in a single
project:

```sh
verune -c .sec.ver.ron switch runtime 1.0.0
export VER_CONFIG=.thr.ver.ron
verune switch runtime 1.0.0
```

When you want to finally start using the runtimes you have in your
configuration, you can use the `scope` subcommand to run a program that has
access to each runtime's version directory. By default, this subcommand will
use the system's command line shell (e.g.
[Bash](https://www.gnu.org/software/bash/)), but other programs and even
arguments can be spawned this way:

```sh
verune scope
$ prog # Success!
$ pkgman # Success!
$ exit
verune scope echo "t" # t
# We can even run runtime programs using this method!
verune scope prog # Success!
verune scope pkgman # Success!
```

### Tips and Tricks

- Running development tools in a `verune` scope is particularly useful for
  compatibility; for example, an IDE can pick up on runtimes and use the
  appropriate runtime versions fairly easily.
- If you're not sure if your setup is ready for a project using `verune`, you
  can run the `check` subcommand to identify if it's safe to use `verune` and
  what issues there may be with your current setup.
- Project configuration files are **recommended** to place in version control.
  They are capable of supporting both organizations and individuals in using
  the exact same version of runtimes.
- Although not recommended, the way `verune` scopes work allows for a behavior
  similar to shims in other version managers, which are aliases to runtime
  programs. This can be done using a shell script that runs the `scope`
  subcommand.
- Much of the business logic of `verune` is provided by `libver`, which is
  under the same licensing and can be used by other projects.

### Licensing

`verune` and `libver` are dual-licensed under the **MIT** and **Apache 2.0**
licenses. Contributions must be licensed in this manner.
