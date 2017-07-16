//! The `action` module provides abstractions for CouchDB-specific HTTP requests
//! and responses.
//!
//! # Summary
//!
//! * An **action** is an HTTP request and response pair.
//!
//! * An action improves type-safety as compared to working with generic HTTP
//!   requests and responses, such as those provided by the
//!   [hyper](https://crates.io/crates/hyper) crate.
//!
//! * However, when using actions, applications can do only what the `couchdb`
//!   crate supports doing.
//!
//! * **TODO:** Provide a means for an application to craft custom requests.
//!
//! * Applications should construct actions by calling the appropriate
//!   [`Client`](../struct.Client.html) method—e.g.,
//!   [`put_database`](../struct.Client.html#method.put_database).
//!
//! # CouchDB API coverage
//!
//! This table shows which parts of the CouchDB API the `couchdb` crate
//! supports.
//!
//! <style type="text/css">
//!  .supported a { font-weight: normal; }
//!  .supported { font-weight: bold; }
//! </style>
//!
//! <table>
//!  <thead>
//!   <tr>
//!    <th>URL path</th>
//!    <th>Method</th>
//!    <th><code>Client</code> method</th>
//!    <th>Description</th>
//!   </tr>
//!  </thead>
//!
//!  <tbody>
//!   <tr>
//!    <td><code>/</code></td>
//!    <td>GET</td>
//!    <td><a href="../struct.Client.html#method.get_root"><code>get_root</code></a></td>
//!    <td>Get server information and other meta-information.</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_all_dbs</code></td>
//!    <td>GET</td>
//!    <td><a href="../struct.Client.html#method.get_all_databases"><code>get_all_databases</code></a></td>
//!    <td>Get a list of all databases on the server.</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="4"><code>/{db}</code></td>
//!    <td>GET</td>
//!    <td><a href="../struct.Client.html#method.get_database"><code>get_database</code></a></td>
//!    <td>Get meta-information about a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>HEAD</td>
//!    <td><a href="../struct.Client.html#method.head_database"><code>head_database</code></a></td>
//!    <td>Test whether a database exists.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td><a href="../struct.Client.html#method.put_database"><code>put_database</code></a></td>
//!    <td>Create a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td><a href="../struct.Client.html#method.delete_database"><code>delete_database</code></a></td>
//!    <td>Delete a database.</td>
//!   </tr>
//!
//!  </tbody>
//! </table>


mod delete_database;
mod get_all_databases;
mod get_database;
mod get_root;
mod head_database;
mod put_database;

pub use self::delete_database::DeleteDatabase;
pub use self::get_all_databases::GetAllDatabases;
pub use self::get_database::GetDatabase;
pub use self::get_root::GetRoot;
pub use self::head_database::HeadDatabase;
pub use self::put_database::PutDatabase;

const E_ACTION_USED: &str = "Cannot use action more than once";
