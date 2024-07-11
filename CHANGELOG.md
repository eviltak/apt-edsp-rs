# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
