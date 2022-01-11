# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/althonos/uniprot.rs/compare/v0.5.0...HEAD


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
