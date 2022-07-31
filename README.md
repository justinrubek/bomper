# bomper

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
