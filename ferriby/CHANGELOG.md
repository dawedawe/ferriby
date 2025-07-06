# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/dawedawe/ferriby/releases/tag/v0.1.1) - 2025-07-06

### Added

- *(app)* Restart the async tasks after chaning the selected source
- *(app)* Some review changes
- *(app)* Add tests for the configfile
- *(app)* Fix key presses on Windows
- *(app)* Try to make config_path Windows compatible
- *(app)* Support custom path for config file
- *(app)* Add support for a config file
- *(app)* Improve UI
- *(app)* Use joinset
- *(app)* First stab at multiple sources
- *(github)* Use http API to also get timestamps from other branches
- *(ui)* Show source in UI
- *(app)* Add animations
- *(app)* Use ASCII art for happiness state
- *(app)* Pass the github repo as a CLI arg
- *(app)* Implement multiple happiness levels
- *(gh)* Make interval dependent on github pat in the env

### Fixed

- *(cargo)* Fix Cargo.toml
- *(git)* Fix some new clippies
- *(app)* Handle network outage more gracefully
- *(docs)* Fix README.md and usage
- *(app)* Fix intervall times

### Other

- *(docs)* Add CHANGELOG.md
- *(docs)* clarify json expectations
- *(docs)* Recommend PATs
- *(app)* Cleanup deps
- Update README.md
- *(docs)* Add vhs
- *(app)* Reduce to one crate
- *(app)* Refactor ticks
- Update ferriby/src/main.rs
- use dedicated tokio tasks for animaion, keyevents and repo checks
- add tests for parse_args
- start parsing multiple sources from args
- *(docs)* Fix README.md
- *(docs)* Document github pat
- Merge pull request #16 from dawedawe/readme
- *(docs)* Put some real content into the README.md
- initial commit

## [0.1.0](https://github.com/dawedawe/ferriby/releases/tag/v0.1.0) - 2025-07-06

### Added

- *(app)* Restart the async tasks after chaning the selected source
- *(app)* Some review changes
- *(app)* Add tests for the configfile
- *(app)* Fix key presses on Windows
- *(app)* Try to make config_path Windows compatible
- *(app)* Support custom path for config file
- *(app)* Add support for a config file
- *(app)* Improve UI
- *(app)* Use joinset
- *(app)* First stab at multiple sources
- *(github)* Use http API to also get timestamps from other branches
- *(ui)* Show source in UI
- *(app)* Add animations
- Add support for a local git repo as a source
- *(app)* Use ASCII art for happiness state
- *(app)* Pass the github repo as a CLI arg
- *(app)* Implement multiple happiness levels
- *(gh)* Make interval dependent on github pat in the env

### Fixed

- *(app)* Handle network outage more gracefully
- *(docs)* Fix README.md and usage
- *(app)* Fix intervall times

### Other

- *(app)* Cleanup deps
- *(app)* Reduce to one crate
- *(app)* Refactor ticks
- Update ferriby/src/main.rs
- use dedicated tokio tasks for animaion, keyevents and repo checks
- add tests for parse_args
- start parsing multiple sources from args
- initial commit
