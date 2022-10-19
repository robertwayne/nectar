# nectar

`nectar` is a Tokio codec providing partial Telnet encoding/decoding with MUD
protocol extension support. It was designed specifically for use with
**[Blossom](https://github.com/robertwayne/blossom)**, but could be inserted
into any Tokio-based application.

## Usage

You need to use Tokio utils with the  `codec` feature enabled, then simply pass
the `TelnetCodec` to the `Framed` struct. See the **[Tokio
docs](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html)**
or the Blossom implementation
**[here](https://github.com/robertwayne/blossom/blob/main/crates/blossom_core/src/connection.rs)**
and
**[here](https://github.com/robertwayne/blossom/blob/main/crates/blossom_core/src/telnet_handler.rs)**.

## License

nectar source code is dual-licensed under either

- **[MIT License](/docs/LICENSE-MIT)**
- **[Apache License, Version 2.0](/docs/LICENSE-APACHE)**

at your option.
