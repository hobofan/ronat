# Ronat

Check your comments for spelling errors!

**This project is in a very early stage, any feedback is appreciated!**

## Usage

As a general rule `ronat` will try to work with the first nightly release following the latest stable version.

The version currently tested against is: **nightly-2016-12-19**

### Optional dependency

If you want to make `ronat` an optional dependency, you can do the following:

In your `Cargo.toml`:

```toml
[dependencies]
ronat = {version = "*", optional = true}

[features]
default = []
```

And, in your `main.rs` or `lib.rs`:

```rust
#![cfg_attr(feature="ronat", feature(plugin))]
#![cfg_attr(feature="ronat", plugin(ronat))]
```

Then build by enabling the feature: `cargo build --features "ronat"`

## Configuration

Using the default `en` dictionary, which doesn't recognize a lot of common Rust and programing terms, you will get a lot of false positives.
To prevent this you can set up a `.aspell.en.pws` file in your HOME directory.

You can find a example file to use for that under [example/.aspell.en.pws](./example/.aspell.en.pws)

## Acknowledgements

Thank you to:
- [rust-clippy](https://github.com/Manishearth/rust-clippy) for being a good entry point to ther world of linters in Rust
- [This PR to the Rust Book](https://github.com/rust-lang/book/pull/338) for the idea

The project is named after linguist [Mitsou Ronat](https://en.wikipedia.org/wiki/Mitsou_Ronat).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
