# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### 🚀 Features

- *(app)* Add gitlab support

### ⚙️ Miscellaneous Tasks

- *(CI)* Remove release-plz, use git-cliff
- *(App)* Refactor some config parsing code
- *(app)* Refactor requests for reuse
- *(docs)* Add config path on Windows
- *(docs)* Update changelog for 0.3.0

## [0.2.0] - 2025-07-11

### 🚀 Features

- *(app)* Add support for codeberg sources
- *(app)* Show source case in UI
- *(app)* Refactor to use a struct for all intervalls

### 📚 Documentation

- *(readme)* Add installation instructions

### ⚙️ Miscellaneous Tasks

- Release v0.1.1
- *(ci)* Enable clippy as we are public now
- *(app)* Refactor to use a trait
- *(app)* Refactor for reuse
- *(app)* Bump version to 0.2.0
- *(docs)* Update vhs
- *(app)* Avoid an allocation

## [0.1.1] - 2025-07-06

### 🐛 Bug Fixes

- *(cargo)* Fix Cargo.toml

## [0.1.0] - 2025-07-06

### 🚀 Features

- *(gh)* Make interval dependent on github pat in the env
- *(app)* Implement multiple happiness levels
- *(app)* Pass the github repo as a CLI arg
- *(app)* Use ASCII art for happiness state
- Add support for a local git repo as a source
- *(app)* Add animations
- *(ui)* Show source in UI
- *(git)* Support non-active branches
- *(github)* Use http API to also get timestamps from other branches
- *(app)* First stab at multiple sources
- *(app)* Use joinset
- *(app)* Improve UI
- *(app)* Add support for a config file
- *(app)* Support custom path for config file
- *(app)* Try to make config_path Windows compatible
- *(app)* Fix key presses on Windows
- *(app)* Add tests for the configfile
- *(app)* Some review changes
- *(app)* Restart the async tasks after chaning the selected source
- *(ci)* Add release-plz github workflow

### 🐛 Bug Fixes

- *(app)* Fix intervall times
- *(docs)* Fix README.md and usage
- *(app)* Handle network outage more gracefully
- *(git)* Fix some new clippies

### ⚙️ Miscellaneous Tasks

- *(docs)* Put some real content into the README.md
- *(github)* Use reqwest for requests
- *(docs)* Document github pat
- *(docs)* Fix README.md
- *(app)* Refactor ticks
- *(app)* Reduce to one crate
- *(docs)* Add vhs
- *(app)* Cleanup deps
- *(docs)* Recommend PATs
- *(ci)* Add ubuntu-latest to os matrix
- *(docs)* Clarify json expectations
- *(docs)* Add CHANGELOG.md

<!-- generated by git-cliff -->
