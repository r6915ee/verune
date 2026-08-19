# Contributing v1.0.0

## Commit/Pull Request Metadata

Commit messages should be formatted as so:

```
scope: short description

Longer description. Consists of zero or more paragraphs. Usually wraps around
at column 80 for words like done here.

Closes: #1
```

The scope can be any of the following:

- `cli`: Changes to the command line interface, alongside all of the logic
  exclusive to `verune`. In other words, this is everything code-related that
  doesn't have anything to do with `libver`.
- `libver`: Changes to miscellaneous things about `libver`, such as source code
  file organization and C headers.
- `kv`: Changes to code responsible for, alongside the semantics, of the
  key-value pair language used by `libver`.
- `runtime`: Changes to the code responsible for retrieving information about
  runtimes.
- `scope`: Changes to the code responsible for opening scopes.
- `docs/<scope>`: Changes to the documentation regarding a certain scope listed
  above.
- `docs`: Changes to any "generic" documentation, primarily the README.
- `build`: Changes to the build system graph (including metadata), _or_
  updating the `.version` file.
- `misc`: Changes to anything not described above.

The `Closes` trailer should reference an issue number that the changes will
close, or a pull request that will be considered merged once the commit lands
on a branch. Other standard trailers work here as well.

Pull requests follow the same scheme. However, the first line of the commit
message is used for the pull request name. The longer description does not
necessarily need to be wrapped at column 80, but it will get wrapped in the
appropriate squash commit, as it will be formatted.

## Branching Tips

Although you do not need to follow these, here are some recommendations for
dealing with branches in pull requests:

- Keep a copy of any target branches handy in both your local copy and the
  remote of your fork. Don't commit anything to these branches. Keep these
  branches in sync with the upstream repository from time to time, especially
  when making a pull request.
- When starting a pull request, place all of the changes you want to make in a
  new branch. This new branch should be based off of the target branch, and it
  will be used for the pull request.
- Pull request branch names can be however you like, but the standard used for
  the upstream repository is to detail the intention of all changes combined
  and then a short description of the actual change in kebab-case, delimited by
  `/`. A basic example would be `fix/spawn-process`.
- Hack away at the pull request branch.

## Generative AI Usage

Please do not read or write to this repository and its metadata using any form
of generative AI. Doing this will result in a ban from the upstream repository
through any means necessary, with no second chances.

The definitions below are as inclusive as they physically can be, and span any
interaction with the repository and its metadata, such as any forums hosted on
Forgejo and the actual code itself. In other words, if it includes any of these
definitions, then it is guaranteed to violate this rule.

- A read constitutes ingesting **any** data provided by the repository using
  **any** form of generative AI.
- A write constitutes writing **any** data into the upstream repository using
  **any** form of generative AI.

In the case of dependencies, although there is an intentionally small amount of
them, if it's found out that any of them violate this rule, then they will
either be removed or replaced.
