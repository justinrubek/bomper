# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [0.6.0](https://github.com/justinrubek/bomper/compare/0.5.1..0.6.0) - 2023-03-18
#### Bug Fixes
- **(nix)** Properly refer to name of package for check - ([d4d41de](https://github.com/justinrubek/bomper/commit/d4d41debf1c84f90d87469b22148fa46a8bbce34)) - [@justinrubek](https://github.com/justinrubek)
- **(nix)** Properly refer to name of package for chheck - ([dd66d14](https://github.com/justinrubek/bomper/commit/dd66d147e048f078fc9de649385ca9add08a557e)) - [@justinrubek](https://github.com/justinrubek)
- disable dry-run by default - ([dfcf09e](https://github.com/justinrubek/bomper/commit/dfcf09ec727163c3734ee985cedac61a9557a4ac)) - [@justinrubek](https://github.com/justinrubek)
#### Build system
- **(nix)** add flake checks - ([6f4299a](https://github.com/justinrubek/bomper/commit/6f4299adbe683271a9fdbd1be032982b13e35684)) - [@justinrubek](https://github.com/justinrubek)
- **(nix)** use crane for building rust packages - ([420c54c](https://github.com/justinrubek/bomper/commit/420c54c9e86eeabe36975286e5ad4a789d8e01e4)) - [@justinrubek](https://github.com/justinrubek)
- update bomper dependency - ([277364b](https://github.com/justinrubek/bomper/commit/277364bc7dfcd604a1aa56c9b5f76095ef934c58)) - [@justinrubek](https://github.com/justinrubek)
- update bomper dependency - ([d453357](https://github.com/justinrubek/bomper/commit/d45335752a8f2c7181d18fba2614ad767de639b4)) - [@justinrubek](https://github.com/justinrubek)
#### Documentation
- **(example)** add examples - ([e87dc92](https://github.com/justinrubek/bomper/commit/e87dc9218688039e70a28f3c57595cd4c1cb9202)) - [@justinrubek](https://github.com/justinrubek)
- **(readme)** clarify examples - ([0e8f45a](https://github.com/justinrubek/bomper/commit/0e8f45a2eef404d4464473332d1f5e0036b85593)) - [@justinrubek](https://github.com/justinrubek)
#### Features
- Cargo.toml updating - ([464ff42](https://github.com/justinrubek/bomper/commit/464ff42992ee8ad79c3cdb2b825131688f024e1f)) - [@justinrubek](https://github.com/justinrubek)
- Autodetect cargo workspace packages - ([96590cb](https://github.com/justinrubek/bomper/commit/96590cbbcc6842c1063209ea6d92b9d10de27de7)) - [@justinrubek](https://github.com/justinrubek)
- Cargo.lock editing - ([4f97982](https://github.com/justinrubek/bomper/commit/4f97982d6ffeed03f643387c9cf86aa435ae9c8e)) - [@justinrubek](https://github.com/justinrubek)
- display program information: version, author, and about - ([0bceca7](https://github.com/justinrubek/bomper/commit/0bceca7067191cce6abe8ecea7033a9050996e12)) - [@justinrubek](https://github.com/justinrubek)
#### Miscellaneous Chores
- **(nix)** Update flake structure - ([2f8004f](https://github.com/justinrubek/bomper/commit/2f8004f82ca0bfeedcfc8654fbe243a2ba5851e3)) - [@justinrubek](https://github.com/justinrubek)
- **(nix/devShell)** add direnv - ([c97b23c](https://github.com/justinrubek/bomper/commit/c97b23c94591c40d4dfb7644d5a7a1695ca5902e)) - [@justinrubek](https://github.com/justinrubek)
- **(nix/devShell)** add bacon - ([243031b](https://github.com/justinrubek/bomper/commit/243031b91789e2f7e4012f2c26b61631ef4434a9)) - [@justinrubek](https://github.com/justinrubek)
- remove needless waiting for input - ([0594e65](https://github.com/justinrubek/bomper/commit/0594e65b84e41ce12eefbd6a4817dfdf17a7d0b3)) - [@justinrubek](https://github.com/justinrubek)
- update cog changelog format - ([adf3689](https://github.com/justinrubek/bomper/commit/adf3689080570c21d80fe66bbbae54c468998439)) - [@justinrubek](https://github.com/justinrubek)
#### Style
- **(nix)** format with alejandra - ([520a41c](https://github.com/justinrubek/bomper/commit/520a41cfacda4b0e6b6e23466bfdb50d64f012e7)) - [@justinrubek](https://github.com/justinrubek)
- cargo fmt - ([d2c10b1](https://github.com/justinrubek/bomper/commit/d2c10b17d073c39f6b2ed97d059f76fe1ce8a9b2)) - [@justinrubek](https://github.com/justinrubek)
#### Tests
- test cargo replacement - ([7312f0a](https://github.com/justinrubek/bomper/commit/7312f0ade1d1337ef5d1d8fd635671411923a798)) - [@justinrubek](https://github.com/justinrubek)
- dual_replace test fails when SearchReplacer causes no overwriting - ([7dab549](https://github.com/justinrubek/bomper/commit/7dab549e23fb3f3c8d113593b395b75e4b9c3408)) - [@justinrubek](https://github.com/justinrubek)

- - -

## 0.5.1 - 2022-08-07
#### Bug Fixes
- SearchReplacer now writes end of segment when there are multiple replacements in a single file - (0c8b410) - Justin Rubek
#### Build system
- **(flake.nix)** refactor to use flake-parts - (c93af18) - Justin Rubek
- update dependency to bomper - (78c93a4) - Justin Rubek
- self-reference flake as dependency for bumping version - (7767bc2) - Justin Rubek
#### Continuous Integration
- **(bomp.toml)** added search checking to all files - (d8ce155) - Justin Rubek
#### Documentation
- **(readme)** explain that bomper uses itself for - (4472b31) - Justin Rubek
#### Style
- clippy - (151c4a4) - Justin Rubek
#### Tests
- Added FileJail and tested SearchReplacer - (6276ed6) - Justin Rubek

- - -

## 0.5.0 - 2022-08-07
#### Build system
- **(bomp.toml)** add Cargo.lock - (a402743) - Justin Rubek
#### Continuous Integration
- **(Cargo.lock)** update bomper version - (5d7dd93) - Justin Rubek
#### Documentation
- **(readme)** added basic description of bomper and uses - (3e25b3c) - Justin Rubek
- describe persist function - (0701bb9) - Justin Rubek
#### Features
- **(bomp.toml)** switched file format to support search and simple replacement - (43182b9) - Justin Rubek
- Added error type for anyhow - (3e27945) - Justin Rubek
- Replacer trait - (d56c711) - Justin Rubek
- SearchReplacer - replaces matched strings only when a verification regex finds a match - (56d38ab) - Justin Rubek
- persist impl for FileReplacer which automatically persists that file - (d7446a2) - Justin Rubek
#### Refactoring
- move file replacement logic into replacers submodule - (83c1d47) - Justin Rubek
#### Style
- cargo fmt - (5e81c23) - Justin Rubek

- - -

## 0.4.5 - 2022-07-31
#### Bug Fixes
- **(cargo)** removed semicolon - (0e1be23) - Justin Rubek
#### Continuous Integration
- **(actions)** added publish-crate workflow with manual steps - (116a204) - Justin Rubek
- **(actions)** removed publish-crates workflow job - (bd63a9e) - Justin Rubek
- **(cog.toml)** added pre_bump_hook to run bomper - (c87a17c) - Justin Rubek
#### Documentation
- **(cargo)** added fields for crates.io - (5be620d) - Justin Rubek
#### Miscellaneous Chores
- **(license)** renamed to not be a markdown file - (7fb00a4) - Justin Rubek

- - -

## 0.4.4 - 2022-07-31
#### Continuous Integration
- made crates.io release action a different step - (22d383f) - Justin Rubek

- - -

## 0.4.3 - 2022-07-31
#### Continuous Integration
- **(actions)** consolidated actions into tag.yml - (7ecfb4a) - Justin Rubek

- - -

## 0.4.2 - 2022-07-31
#### Continuous Integration
- **(actions)** made GitHub release occur across 2 steps - (5e3bd29) - Justin Rubek

- - -

## 0.4.1 - 2022-07-31
#### Continuous Integration
- added GitHub actions for publishing releases to crates.io - (6dc27eb) - Justin Rubek
- added Cargo.lock and removed incorrect .gitignore value - (b7abe5a) - Justin Rubek
#### Documentation
- updated README.md with information on nix and the features provided by the default devShell - (9dfca5b) - Justin Rubek
- fixed link to Cocogitto in README.md - (8af4f7e) - Justin Rubek

- - -

## 0.4.0 - 2022-07-31
#### Features
- added bomp_files function to handle overwriting multiple files - (4910158) - Justin Rubek
#### Refactoring
- made overwrite_file accept and consume a PathBuf - (970286e) - Justin Rubek

- - -

## 0.3.0 - 2022-07-30
#### Features
- file changes are only applied when all operations succeed - (8a2e53d) - Justin Rubek

- - -

## 0.2.0 - 2022-07-30
#### Continuous Integration
- add bomp.toml - (a9f69a6) - Justin Rubek
#### Documentation
- Add information on cocogitto - (294905b) - Justin Rubek
- Add README - (452ff0c) - Justin Rubek
- Add license - (3d11df6) - Justin Rubek
#### Features
- added cli args with clap - (45e217f) - Justin Rubek
- app replaces files when ran - (1e04875) - Justin Rubek
- added file replacement using regex - (f07806c) - Justin Rubek
- added thiserror type - (f9c1d9a) - Justin Rubek
- made files configuration a set - (97e338b) - Justin Rubek
- load config from bomp.toml - (062b3c3) - Justin Rubek
#### Miscellaneous Chores
- fix clippy issues - (8ca7936) - Justin Rubek
- remove println - (c58bb66) - Justin Rubek

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).