# CouchDB

[![Build Status](https://travis-ci.org/couchdb-rs/couchdb.svg?branch=master)](https://travis-ci.org/couchdb-rs/couchdb)

---

**This project is deprecated.** A new crate, [Chill][chill-rs], is the
way forward and will provide a safer and more useful abstraction, as
well as eliminating some inefficiencies. You can read a little bit about
the rationale [here][rethinking_couchdb_in_rust].

At the time of writing this (2016-03-26), Chill has minimal coverage of
the CouchDB API. I expect Chill to catch up to and surpass this crate in
coverage by late spring of 2016.

---

This project provides a CouchDB client-side library for the Rust
programming language.

The library provides low-level access to individual HTTP actions—e.g.,
PUT database, GET document, etc. It handles the menial task of sending
requests and receiving responses, thereby allowing application
programmers to focus on their business logic.

## Project roadmap

The latest release is **v0.5.1**, which was released **2016-02-12**.

* [v0.5.1 change log](https://github.com/couchdb-rs/couchdb/blob/v0.5.1/CHANGELOG.md).
* [v0.5.1 documentation](https://couchdb-rs.github.io/couchdb/doc/v0.5.1/couchdb/index.html).
* [v0.5.1 issues](https://github.com/couchdb-rs/couchdb/issues?q=milestone%3Av0.5.1).
* [v0.5.1 crates.io page](https://crates.io/crates/couchdb/0.5.1).

**There is no plan to make another release.**

## License

CouchDB-rs is licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0), or
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT).

## Feedback

Do you find this crate useful? Not useful? [Please send
feedback!](mailto:c.m.brandenburg@gmail.com)

[chill-rs]: https://github.com/chill-rs/chill
[rethinking_couchdb_in_rust]: https://cmbrandenburg.github.io/post/2016-02-23-rethinking_couchdb_in_rust/
