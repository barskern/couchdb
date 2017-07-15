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
//!    <td rowspan="3"><code>/{db}</code></td>
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
mod head_database;
mod put_database;

pub use self::delete_database::DeleteDatabase;
pub use self::head_database::HeadDatabase;
pub use self::put_database::PutDatabase;

const E_ACTION_USED: &str = "Cannot use action more than once";
