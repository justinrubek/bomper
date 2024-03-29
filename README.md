# bomper

bomper is a CLI tool that replaces the contents of multiple files specified by `bomp.ron`.
The intended use is to update version strings that are hardcoded into project files.
A `bomp.ron` will be provided by the project containing paths to files that need to be updated.
If the operation fails then no files will be changed.

This can then be combined with a separate tool, such as a [pre bump hook](https://docs.cocogitto.io/guide/#bump-hooks) in Cocogitto's `cog.toml`.

## usage

Run the command with the args `--help` to view the instructions.

See `./examples` for some specific examples. Additionally, see bomper's `cog.toml` and `bomp.ron` for an integration with cocogitto.
When running `cog bump,` bomper is invoked and is used to  update the files as part of the version bump commit.

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
