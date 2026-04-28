# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-04-25

### Added
- WebAssembly binding (`syllabify-fr-wasm`) with `@dyscolor/syllabify-fr-wasm` npm package
- C FFI binding (`syllabify-fr-ffi`)
- Inter-word liaison predicates (`liaisonAmont` / `liaisonAval`)
- HTML rendering with syllable spans and liaison markers
- Homograph disambiguation in `syllabify_text`

### Changed
- Hyphens and underscores kept in-word (LC6 v6 change)

[Unreleased]: https://github.com/kevinlabory/syllabify-fr/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/kevinlabory/syllabify-fr/releases/tag/v0.4.0
