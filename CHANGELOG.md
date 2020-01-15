# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/althonos/uniprot.rs/compare/v0.1.1...HEAD


## [v0.1.1] - 2020-15-01

### Changed
- Removed remaining explicit [`panic!`] calls.
### Added
- [`FromStr`] implementation for some enum types that are read from XML
  attributes.

[v0.1.1]: https://github.com/althonos/uniprot.rs/compare/v0.1.0...v0.1.1
[`panic!`]: https://doc.rust-lang.org/std/macro.panic.html
[`FromStr`]: https://doc.rust-lang.org/std/str/trait.FromStr.html


## [v0.1.0] - 2020-15-01

Initial release.

[v0.1.0]: https://github.com/althonos/uniprot.rs/compare/52e6940...v0.1.0
