# bomper

[<img alt="github" src="https://img.shields.io/badge/github-justinrubek/bomper-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/justinrubek/bomper)
[<img alt="crates.io" src="https://img.shields.io/crates/v/bomper.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/bomper)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-bomper-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/bomper)

`bomper` is a one-stop shop CLI tool that can update version strings in project files and maintain your project's changelog.

- automatically bump [semver](https://semver.org/) strings in project files
- changelog generation based on [conventional commits](https://www.conventionalcommits.org/)
- no system-level dependencies: bomper produces a statically compiled binary thanks to [gitoxide](https://github.com/byron/gitoxide)

## foreword

bomper was originally created to be used as a hook in [cocogitto](https://github.com/cocogitto/cocogitto).
cocogitto does not update version strings in project files, so bomper was created to fill that gap.
Their implementation changelog implementation is more full-featured and configurable than bomper, so
for projects that require this flexibility in changelog generation, consider using cocogitto instead.
You can still use bomper as a hook without the changelog generation.

bomper aims to be a simpler, more focused, and opinionated tool that can be customized to fit your needs.

## usage

Run the command with `--help` to view the instructions.

In order for bomper to work, it must be told where to find the files that need to be updated.
The configuration supports either arbitrary files or a number of supported project types (currently only `cargo`, but this can be added to).
This configuration is specified in a `bomp.ron` file.
The file can be in the root-level of the project, in the `${PRJ_CONFIG_HOME}` directory, or specified with the `--config-file` flag.
See `./examples` for some specific examples of configuration files.

There are three main commands: `bump`, `changelog`, and `raw-bump`.
The `bump` command will update the version strings in the files specified by the `bomp.ron` file, add changes to the changelog, create a commit, and tag the changes.
The `changelog` command will generate and display a changelog based on the commit messages in the repository, but will not update any files.
The `raw-bump` command will update the version strings in the files specified by the `bomp.ron` file, but will not add changes to the changelog, commit, or tag the changes.

### flake module

A [flake-parts](https://flake.parts) module is provided to allow integration of bomper without needing to keep a `bomp.ron` file in your project's files.
It works by creating a script that wraps the `bomper` command and passes the `bomp.ron` file as an argument.
The configuration to use is specified as a nix module option so that it can live inside your nix configuration.
To use this you must use `flake-parts`, import the flake module, and specify the options in a `perSystem` block.
See [`./flake-parts/bomper.nix`](./flake-parts/bomper.nix) for an example of its usage - this project dogfoods the module.

With the module enabled, you can access the wrapped bomper package using the `config` parameter of a `perSystem` block: `config.bomper.wrappedBomper`.

## Contributing

The main dependency for working on this project is nix.
To work on this project, first ensure that [Nix: the package manager](https://nixos.org/download.html) is available on your system.
A `flake.nix` is provided for configuring a development environment.
To use this, enter a dev shell: `nix develop`
