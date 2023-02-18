# nectar

`nectar` is a Tokio codec providing a partial Telnet protocol (RFC 854)
implementation.

Supports primary negotiation options: DO, DONT, WILL, WONT. Supports
subnegotiation (NAWS, custom byte sequences). Aims to implement some of the
popular MUD protocol extensions.

## Usage

You must be using `tokio-utils` with the  `codec` feature enabled. In your
stream handler, pass the `TelnetCodec` to the `Framed` struct along with your
stream. See the **[Tokio
docs](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html)**
for more information on how this is used.

In general, this is all you need to know. Your stream will now be interpreted as
Telnet.

## Example

See the **[echo_server](/examples/echo_server)** directory for a working example
with commented code. If you have cloned the `nectar` repository,  you can run
the echo server with cargo and then connect with `telnet localhost 5000`.

*Note: Make sure you check the dependencies in the example `Cargo.toml` file.*

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    let listener = TcpListener::bind(addr).await?;

    println!("telnet server started on: {}", addr);

    loop {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = handler(stream).await {
                    eprintln!("error: {}", e);
                }
            });
        }
    }
}

async fn handler(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Wrap the stream with the TelnetCodec. All the encoding/decoding is
    // handled by the codec, so you can now match for events!
    let mut frame = Framed::new(stream, TelnetCodec::new(1024));

    while let Some(Ok(msg)) = frame.next().await {
        match msg {
            // We'll keep it simple and only match against the Message event.
            TelnetEvent::Message(string) => {
                // Let's echo back what we received.
                frame.send(TelnetEvent::Message(string)).await?;
            }
            _ => break,
        }
    }
}
```

You can check out the **[Blossom](https://github.com/robertwayne/blossom)**
source code for an example of `nectar` in a more complex, real-world scenario.

## License

nectar source code is dual-licensed under either

- **[MIT License](/docs/LICENSE-MIT)**
- **[Apache License, Version 2.0](/docs/LICENSE-APACHE)**

at your option.
