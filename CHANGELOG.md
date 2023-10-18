# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]
[Unreleased]: https://github.com/althonos/uniprot.rs/compare/v0.7.0...HEAD


## [v0.7.0] - 2023-10-18
[v0.7.0]: https://github.com/althonos/uniprot.rs/compare/v0.6.0...v0.7.0

### Added
- `smartstring` feature for using the `smartstring` crate to reduce heap allocations.

### Changed
- Reduce default sleep duration to reduce strain on CPU.
- Update `quick-xml` dependency to `v0.30.0`.
- Use a dedicated producer thread to read data from the reader in `ThreadedParser`.

### Fixed
- Broken extraction of names in `Citation::from_xml`

### Removed
- Unused `fnv` dependency.
- Deprecated `uniprot::parse` top-level function.


## [v0.6.0] - 2022-10-17
[v0.6.0]: https://github.com/althonos/uniprot.rs/compare/v0.5.2...v0.6.0

### Removed
- Deprecated `CalciumBindingRegion`, `MetalIonBindingSite` and `NucleotidePhosphateBindingRegion` variants of `uniprot::FeatureType`.

### Added
- `uniprot::Ligand` and `uniprot::LigandPart` structs for the `ligand` and `ligand_part` attributes of `uniprot::Feature`.

### Changed
- Updated URLs in documentation examples to use the new Uniprot REST API.
- Moved the `uniref::parse_entry` example to` uniref::parse` since single-entry requests to UniRef don't return single entries anymore.


## [v0.5.2] - 2022-02-28
[v0.5.2]: https://github.com/althonos/uniprot.rs/compare/v0.5.1...v0.5.2

### Added
- `PartialEq`, `Eq`, `Hash` and `Clone` traits to simple enum types (like `uniprot::uniprot::FeatureType`).

### Changed
- Feature gate the `url` crate dependency to skip parsing links into `url::Url` if not needed.
- Remove dependency on `thiserror` by manually implementing `std::error::Error` where needed.


## [v0.5.1] - 2022-01-11
[v0.5.1]: https://github.com/althonos/uniprot.rs/compare/v0.5.0...v0.5.1

### Fixed
- Large test files being included in distributed `crates.io` source package.


## [v0.5.0] - 2022-01-11
[v0.5.0]: https://github.com/althonos/uniprot.rs/compare/v0.4.0...v0.5.0

### Added
- `parse_entry` functions to parse a single UniProt, UniParc or UniRef entry.

### Fixed
- Parsing of creation dates with a defined timezone (e.g. `2021-01-11Z`).


## [v0.4.0] - 2021-07-24
[v0.4.0]: https://github.com/althonos/uniprot.rs/compare/v0.3.1...v0.4.0

### Added
- `uniprot::uniref` module to parse UniRef XML files.
- `uniprot::uniparc` module to parse UniParc XML files.

### Changed
- Moved types to parse UniProt XML files to the `uniprot::uniprot` module.

### Fixed
- Parsers now check the name of the root element before starting to parse the entries.


## [v0.3.1] - 2020-01-19
[v0.3.1]: https://github.com/althonos/uniprot.rs/compare/v0.3.0...v0.3.1

### Changed
- `lazy_static` and `num_cpus` are only required ot build with `threading` feature.
- Slightly improved performance of `ThreadedParser`.


## [v0.3.0] - 2020-01-19
[v0.3.0]: https://github.com/althonos/uniprot.rs/compare/v0.2.0...v0.3.0

### Added
- [`ThreadedParser::with_threads`](https://docs.rs/uniprot/latest/uniprot/parser/struct.ThreadedParser.html#methods) constructor to control the number of threads to spawn when parsing
### Changed
- `ThreadedParser` does not required the reader to be `Send + 'static` anymore.


## [v0.2.0] - 2020-01-18
[v0.2.0]: https://github.com/althonos/uniprot.rs/compare/v0.1.1...v0.2.0

### Added
- Implemented multithreading parser using [`crossbeam-channel`](https://docs.rs/crossbeam-channel), which can be removed by disabling the `threading` feature.
- Improved documentation of `::error` and `::parser` modules.
### Fixed
- Missing implementation of `submittedName` deserialization within `protein` entries that crashed on TrEMBL.


## [v0.1.1] - 2020-01-15
[v0.1.1]: https://github.com/althonos/uniprot.rs/compare/v0.1.0...v0.1.1

### Changed
- Removed remaining explicit [`panic!`](https://doc.rust-lang.org/std/macro.panic.html) calls.
### Added
- [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) implementation for some enum types that are read from XML attributes.


## [v0.1.0] - 2020-01-15
[v0.1.0]: https://github.com/althonos/uniprot.rs/compare/52e6940...v0.1.0

Initial release.
