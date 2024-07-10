# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/eviltak/apt-edsp/releases/tag/v0.1.0) - 2024-07-10

### Other
- Add dependabot config
- Add release-plz workflow
- Add build and test workflow
- Use Bool where appropriate
- Support missing/default value serialization and deserialization
- Add Bool newtype to serialize "yes" and "no" to a bool
- Move serde test case utilities to test_util test module
- Reformat code
- Add RustRover files
- Add Installed and Conflicts fields to Package
- Use Version and Dependency structs in Package model
- Add util submodule with TryFromStringVisitor deserialization visitor
- Split into submodules
- Add package dependencies model
- Add package relationships model
- Add Version model
- Add models for EDSP scenario (input) and response (output)
- Add Cargo.toml
- Add .gitignore
