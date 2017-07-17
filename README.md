# CouchDB

[![Build Status](https://travis-ci.org/couchdb-rs/couchdb.svg?branch=master)](https://travis-ci.org/couchdb-rs/couchdb)

---

**This project is reborn!**

As of its v0.6.0 release, the `couchdb` crate has new life as a toolkit
instead of providing a full-blown client.

In a nutshell, the `couchdb` crate now provides passive,
“building-block” types for working with CouchDB in Rust. Applications
may use as few or as many of these types as makes the most sense. Actual
HTTP communication with a CouchDB server is now accomplished by some
other means, such as [hyper][hyper_crate] or [reqwest][reqwest_crate].

## Project roadmap

The latest release is **v0.6.0**, which was released **2017-07-17**.

* [v0.6.0 change log](https://github.com/couchdb-rs/couchdb/blob/v0.6.0/CHANGELOG.md).
* [v0.6.0 documentation](https://couchdb-rs.github.io/couchdb/doc/v0.6.0/couchdb/index.html).
* [v0.6.0 issues](https://github.com/couchdb-rs/couchdb/issues?q=milestone%3Av0.6.0).
* [v0.6.0 crates.io page](https://crates.io/crates/couchdb/0.6.0).

The next release is expected to be **v0.6.1** and has no schedule.

* [master change log](https://github.com/couchdb-rs/couchdb/blob/master/CHANGELOG.md).
* [v0.6.1 issues](https://github.com/couchdb-rs/couchdb/issues?q=milestone%3Av0.6.1).

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
[hyper_crate]: https://crates.io/crates/hyper
[reqwest_crate]: https://crates.io/crates/reqwest
[rethinking_couchdb_in_rust]: https://cmbrandenburg.github.io/post/2016-02-23-rethinking_couchdb_in_rust/
