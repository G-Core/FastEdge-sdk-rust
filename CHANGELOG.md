# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/G-Core/FastEdge-sdk-rust/compare/v0.3.1...v0.3.2) - 2025-11-21

### Added

- implement Display trait for Error and add user diagnostics utility

### Other

- Update src/proxywasm/utils.rs
- Update src/lib.rs
- rename utils module to helper and update references

## [0.3.1](https://github.com/G-Core/FastEdge-sdk-rust/compare/v0.3.0...v0.3.1) - 2025-11-04

### Added

- add wit submodule for improved dependency management

### Other

- Update src/proxywasm/key_value.rs
- rename zrange to zrange_by_score and update related documentation

## [0.2.1](https://github.com/G-Core/FastEdge-sdk-rust/compare/v0.2.0...v0.2.1) - 2025-10-09

### Added

- proxy-wasm dictionary interface
- adding new key-value methods
- key-value wit interface

### Fixed

- switch-toggle version
- replaced cuckoo filter with bloom
- move to a file release-plz
- added scan and zscan docs
- added key-value store proxywasm interface
- changing score type to f64
- changing interface names to more redis like
- remove key-value bloom filter interface
- key-value example
- simplified the interface for -by-prefix methods
- improving documentations

### Other

- Merge pull request #40 from G-Core/fix/proxywasm_key_value_error_mapping
- Update src/proxywasm/key_value.rs

## [0.1.6](https://github.com/G-Core/FastEdge-sdk-rust/compare/v0.1.5...v0.1.6) - 2024-05-10

### Other
- Merge pull request [#18](https://github.com/G-Core/FastEdge-sdk-rust/pull/18) from G-Core/update_wasmtime

## [0.1.5](https://github.com/G-Core/FastEdgeSDK/releases/tag/fastedge-v0.1.5) - 2024-05-08

### Fixed
- fix rust sdk README.md link
- fix workspace member versions

### Other
- add release action
- update wit_bindgen to latest version
- refactored to mono rust sdk and add classification-nn-demo
- Added watermark Rust example ([#8](https://github.com/G-Core/FastEdgeSDK/pull/8))
- add wasi-nn wit binding
- forward wasi-nn spec
- set github public hosted runner
- Copyright update and description
- bump patch version
- readme description
- Create README.md
- added HTTP Method Options to supported method list
- renamed rust dir to fastedge-rust-sdk
- allow semver dependencies
- add rust workspace cargo
- first version of rust sdk
- Initial commit
