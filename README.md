# bomper

bomper is a CLI tool that replaces the contents of multiple files specified by `bomp.ron`.
The intended use is to update version strings that are hardcoded into project files.
A `bomp.ron` will be provided by the project containing paths to files that need to be updated.
If the operation fails then no files will be changed.

This can then be combined with a separate tool, such as a [pre bump hook](https://docs.cocogitto.io/guide/#bump-hooks) in Cocogitto's `cog.toml`.

## usage

Run the command with the args `--help` to view the instructions.

See `./examples` for some specific examples. Additionally, see bomper's `cog.toml` for an integration with cocogitto.
When running `cog bump,` bomper is invoked and is used to  update the files as part of the version bump commit.

### flake module

A [flake-parts](https://flake.parts) module is provided to allow integration of bomper without needing to keep a `bomp.ron` file in the project's files.
It works by creating a script that wraps the `bomper` command and passes the `bomp.ron` file as an argument.
The configuration to use is specified as a nix module option so that it can live inside your nix configuration.
To use this you must use `flake-parts`, import the flake module, and specify the options in a `perSystem` block.
See [`./flake-parts/bomper.nix`](./flake-parts/bomper.nix) for an example of its usage - this project dogfoods the module.

With the module enabled, you can access the wrapped bomper package using the `config` parameter of a `perSystem` block: `config.bomper.wrappedBomper`.

## Contributing

The main dependency for working on this project is nix.
To work on this project first ensure that [Nix: the package manager](https://nixos.org/download.html) is available on your system.
A `flake.nix` is provided for configuring a development environment.
To use this, enter a dev shell: `nix develop` 

### dev shell

The provided shell will include the dependencies needed for development.

#### features
- [Cocogitto](https://github.com/cocogitto/cocogitto) is included to generate a CHANGELOG.md based on commit messages
- [pre-commit-hooks.nix](https://github.com/cachix/pre-commit-hooks.nix) are configured to enforce code formatting
    - [rustfmt](https://github.com/rust-lang/rustfmt) is included to allow using the `cargo fmt` command
- [bomper](https://github.com/justinrubek/bomper) is used to update hardcoded version strings in files during `cog bump`
