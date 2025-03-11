# shapely

[![crates.io](https://img.shields.io/crates/v/shapely.svg)](https://crates.io/crates/shapely)
[![documentation](https://docs.rs/shapely/badge.svg)](https://docs.rs/shapely)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/shapely.svg)](./LICENSE)

A Rust reflection, introspection, serialization and deserialization framework.

A single, lightweight derive macro (thanks to [unsynn](https://crates.io/crates/unsynn))

```rust
#[derive(Shapely)]
struct Blah {
    foo: u32,
    bar: String,
}
```

...gives you the equivalent of `Debug`, `Serialize`, `Deserialize`, and more.

shapely's approach is halfway between [serde](https://crates.io/crates/serde) and [bevy_reflect](https://crates.io/crates/bevy_reflect)

See the [shapely README](./shapely/README.md) for more info.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
