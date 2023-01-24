//! cacache is a Rust library for managing local key and content address
//! caches. It's really fast, really good at concurrency, and it will never
//! give you corrupted data, even if cache files get corrupted or manipulated.
//!
//! ## API Layout
//!
//! The cacache API is organized roughly similar to `std::fs`; most of the
//! toplevel functionality is available as free functions directly in the
//! `cacache` module, with some additional functionality available through
//! returned objects, as well as `WriteOpts`, which is analogous to
//! `OpenOpts`, but is only able to write.
//!
//! ### Suffixes
//!
//! You may notice various suffixes associated with otherwise familiar
//! functions:
//!
//! * `_hash` - Since cacache is a content-addressable cache, the `_hash`
//!   suffix means you're interacting directly with content data, skipping the
//!   index and its metadata. These functions use an `Integrity` to look up
//!   data, instead of a string key.
//!
//! ## Examples
//!
//! ```no_run
//! fn main() -> cacache_sync::Result<()> {
//!   // Data goes in...
//!   cacache_sync::write("./my-cache", "key", b"hello")?;
//!
//!   // ...data comes out!
//!   let data = cacache_sync::read("./my-cache", "key")?;
//!   assert_eq!(data, b"hello");
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Lookup by hash
//!
//! What makes `cacache` content addressable, though, is its ability to fetch
//! data by its "content address", which in our case is a ["subresource
//! integrity" hash](https://crates.io/crates/ssri), which `cacache_sync::put`
//! conveniently returns for us. Fetching data by hash is significantly faster
//! than doing key lookups:
//!
//! ```no_run
//! fn main() -> cacache_sync::Result<()> {
//!   // Data goes in...
//!   let sri = cacache_sync::write("./my-cache", "key", b"hello")?;
//!
//!   // ...data gets looked up by `sri` ("Subresource Integrity").
//!   let data = cacache_sync::read_hash("./my-cache", &sri)?;
//!   assert_eq!(data, b"hello");
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Large file support
//!
//! `cacache-sync` supports large file reads, through an API reminiscent of
//! `std::fs::OpenOptions`:
//!
//! ```no_run
//! use std::io::{Read, Write};
//!
//! fn main() -> cacache_sync::Result<()> {
//!   let mut fd = cacache_sync::Writer::create("./my-cache", "key")?;
//!   for _ in 0..10 {
//!     fd.write_all(b"very large data").expect("Failed to write to cache");
//!   }
//!   // Data is only committed to the cache after you do `fd.commit()`!
//!   let sri = fd.commit()?;
//!   println!("integrity: {}", &sri);
//!
//!   let mut fd = cacache_sync::Reader::open("./my-cache", "key")?;
//!   let mut buf = String::new();
//!   fd.read_to_string(&mut buf).expect("Failed to read to string");
//!
//!   // Make sure to call `.check()` when you're done! It makes sure that what
//!   // you just read is actually valid. `cacache` always verifies the data
//!   // you get out is what it's supposed to be. The check is very cheap!
//!   fd.check()?;
//!
//!   Ok(())
//! }
//! ```

pub use serde_json::Value;
pub use ssri::Algorithm;

mod content;
mod errors;
mod index;

mod get;
mod ls;
mod put;
mod rm;

pub use errors::{Error, Result};
pub use index::Metadata;

pub use get::*;
pub use ls::*;
pub use put::*;
pub use rm::*;
