# cacache-sync ![CI](https://github.com/kade-robertson/cacache-rs-sync/workflows/CI/badge.svg) ![crates.io](https://img.shields.io/crates/v/cacache-sync.svg)

A high-performance, concurrent, content-addressable disk cache, with only sync APIs.

## Notes

This is a fork of the `cacache` crate here: https://github.com/zkat/cacache-rs,
which removes all async code and dependencies, and makes the sync APIs the
primary usage (no need for `_sync` suffixes).

The motivation is pretty simple -- in another project I was using `cacache`,
where I only relied on the sync API. I seem to still be paying the cost of
having async-related dependencies, so this fork is mostly for people who are
definitely only going to be using the sync APIs. I'm not sure how translatable
these savings are across other projects, but compile times for debug builds
dropped by ~33%.

This fork is likely going to be minimally supported -- I don't see much needing
to change about the sync implementations here. If you want to see changes here,
you should probably push those to the original project (and consider supporting
it as well).

## Example

```rust
use cacache_sync;

async fn main() -> Result<(), cacache_sync::Error> {
    let dir = String::from("./my-cache");

    // Write some data!
    cacache_sync::write(&dir, "key", b"my-async-data")?;

    // Get the data back!
    let data = cacache_sync::read(&dir, "key")?;
    assert_eq!(data, b"my-async-data");

    // Clean up the data!
    cacache_sync::clear(&dir)?;
}
```

## Install

Using [`cargo-edit`](https://crates.io/crates/cargo-edit)

`$ cargo add cacache-sync`

Minimum supported Rust version is `1.66.1`.

## Documentation

- [API Docs](https://docs.rs/cacache-sync)

## Features

- Sync APIs are the primary API.
- `std::fs`-style API
- Extraction by key or by content address (shasum, etc)
- [Subresource Integrity](#integrity) web standard support
- Multi-hash support - safely host sha1, sha512, etc, in a single cache
- Automatic content deduplication
- Atomic content writes even for large data
- Fault tolerance (immune to corruption, partial writes, process races, etc)
- Consistency guarantees on read and write (full data verification)
- Really helpful, contextual error messages
- Large file support
- Pretty darn fast
- Arbitrary metadata storage
- Cross-platform: Windows and case-(in)sensitive filesystem support
- Punches nazis

## Contributing

The cacache team enthusiastically welcomes contributions and project participation! There's a bunch of things you can do if you want to contribute! The [Contributor Guide](CONTRIBUTING.md) has all the information you need for everything from reporting bugs to contributing entire new features. Please don't hesitate to jump in if you'd like to, or even ask us questions if something isn't clear.

All participants and maintainers in this project are expected to follow [Code of Conduct](CODE_OF_CONDUCT.md), and just generally be excellent to each other.

Happy hacking!

## License

This project is licensed under [the Apache-2.0 License](LICENSE.md).
