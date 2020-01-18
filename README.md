# `uniprot.rs` [![Star me](https://img.shields.io/github/stars/althonos/uniprot.rs.svg?style=social&label=Star&maxAge=3600)](https://github.com/althonos/uniprot.rs/stargazers)

*Rust data structures and parser for the [UniprotKB database(s)].*

[UniprotKB database(s)]: https://www.uniprot.org/

[![TravisCI](https://img.shields.io/travis/althonos/uniprot.rs/master.svg?maxAge=600&style=flat-square)](https://travis-ci.com/althonos/uniprot.rs/branches)
[![Codecov](https://img.shields.io/codecov/c/gh/althonos/uniprot.rs/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/althonos/uniprot.rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs)
[![Crate](https://img.shields.io/crates/v/uniprot.svg?maxAge=600&style=flat-square)](https://crates.io/crates/uniprot)
[![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/uniprot)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs/blob/master/CHANGELOG.md)
[![GitHub issues](https://img.shields.io/github/issues/althonos/uniprot.rs.svg?style=flat-square&maxAge=600)](https://github.com/althonos/uniprot.rs/issues)


## Usage

The `uniprot::parse` function can be used to obtain an iterator over the entries
of a UniprotKB database in XML format (either SwissProt or TrEMBL).

```rust
extern crate uniprot;

let f = std::fs::File::open("tests/uniprot.xml")
   .map(std::io::Buffer::new)
   .unwrap();

for r in uniprot::parse(f) {
   let entry = r.unwrap();
   // ... process the Uniprot entry ...
}
```

Any [`BufRead`](https://doc.rust-lang.org/stable/std/io/trait.BufRead.html)
implementor can be used as an input, so the database files can be streamed
directly from their [online location](https://www.uniprot.org/downloads) with
the help of an HTTP library such as [`reqwest`](https://docs.rs/reqwest), or
using the [`ftp`](https://docs.rs/ftp) library.

See the online documentation at [`docs.rs`](https://docs.rs/uniprot) for more
examples, and some details about the different features available.


## Features

- [`threading`] (_**enabled** by default_): compiles the multithreaded parser
  that offers a 90% speed increase when processing XML files.

## Changelog

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
and provides a [changelog](https://github.com/althonos/uniprot.rs/blob/master/CHANGELOG.md)
in the [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) format.
