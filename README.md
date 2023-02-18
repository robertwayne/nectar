# nectar

`nectar` is a Tokio codec providing a partial Telnet protocol (RFC 854)
implementation.

Supports primary negotiation options: DO, DONT, WILL, WONT. Supports
subnegotiation (NAWS, custom byte sequences). Aims to implement some of the
popular MUD protocol extensions.

## Usage

Simply use ```cargo add nectar``` in your root project directory.

You must also be using `tokio-utils` with the  `codec` feature enabled, as well as
`tokio`.

See the Tokio
**[docs](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html)**
if you wish to dive deeper into codecs. Essentially it is just a way to encode
and decode the underlying TCP Stream in a structured manner - in this case, our
structure being the Telnet protocol.

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

    // Standard Tokio setup - we accept new connections and pass the
    // stream off to our handler function where the real work happens.
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
    // We construct a 'Frame', which is just a wrapper around the underlying
    // stream that is decoded by the `nectar::TelnetCodec`.
    let mut frame = Framed::new(stream, TelnetCodec::new(1024));

    // In a real application, you would want to handle Some(Err(_)) and None
    // variants, but for this example we'll be succinct for simplicities sake.
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
