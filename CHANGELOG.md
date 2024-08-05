# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/eviltak/apt-edsp-rs/compare/v0.2.0...v0.3.0) - 2024-08-05

### Bug Fixes and Improvements
- *(cargo)* Add `repository` metadata
- *(cargo)* Exclude unnecessary files from crate package

### Features
- Add `Progress::write_to` method
- Add `Answer::write_to` method
- [**breaking**] Rename `Scenario::from_read` to `read_from` and return Error
- Derive traits for `Answer` enum
- *(answer)* Add recommended fields to `Install` and `Remove`
- *(answer)* Accurately represent EDSP in `Answer` variants
- *(Progress)* Make all fields of `Progress` public

### Testing
- Add tests for `Answer` enum
- Add `ser_test!` helper macro

## [0.2.0](https://github.com/eviltak/apt-edsp-rs/compare/v0.1.0...v0.2.0) - 2024-08-04

### Bug Fixes
- Remove unnecessary logs

### Documentation
- Document top-level items
- *(Bool)* Remove unresolved links to const generic parameter

### Features
- [**breaking**] Make all fields of all `answer` structs public
- *(Bool)* Implement `From` to convert to and from `bool`

### Refactors
- [**breaking**] Move version mod inside scenario mod
- [**breaking**] Rename response mod to answer and extract progress mod

## [0.1.0](https://github.com/eviltak/apt-edsp-rs/releases/tag/v0.1.0) - 2024-07-11

### Bug Fixes
- *(version)* Manually implement Hash for Version
- *(bool)* Support missing/default value serialization and deserialization

### Features
- *(scenario)* [**breaking**] Use Bool where appropriate
- Add Bool newtype to serialize "yes" and "no" to a bool
- *(scenario)* Add Installed and Conflicts fields to Package
- *(scenario)* [**breaking**] Use Version and Dependency structs in Package model
- *(scenario)* Add package dependencies model
- *(scenario)* Add package relationships model
- *(scenario)* Add Version model
- Add models for EDSP scenario (input) and response (output)

### Internal Changes
- Add private util module with TryFromStringVisitor deserialization visitor

### Licensing
- *(license)* License under BSD-3

### Refactors
- *(scenario)* [**breaking**] Split into submodules

### Styling
- Reformat code

### Testing
- Move serde test case utilities to test_util test module
