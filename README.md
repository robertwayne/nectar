# nectar

`nectar` is a Tokio codec providing partial Telnet encoding/decoding with MUD
protocol extension support. It was designed specifically for use with
**[Blossom](https://github.com/robertwayne/blossom)**, but could be inserted
into any Tokio-based application.

## Usage

You need to use `tokio-utils` with the  `codec` feature enabled, then simply pass
the `TelnetCodec` to the `Framed` struct. See the **[Tokio
docs](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html)** for more information.

```rust
// Example of a simple connection loop running in Tokio.
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use nectar::{event::TelnetEvent, TelnetCodec};

async fn connection_loop(stream: TcpStream) {
    let mut frame = Framed::new(stream, TelnetCodec::new());

    loop {
        tokio::select! {
            result = frame.next() => match result {
                Some(msg) => {
                    // Handle message
                }
                None => {
                    break;
                }
            }
        }
    }

    frame.send_message("Goodbye!".to_string()).await?;
}
```

```rust
// Example of sending an IAC (Interpret-As-Command) message.
// You can see a more realistic example in the Blossom source code here: 
// https://github.com/robertwayne/blossom/blob/dev/blossom/src/auth.rs#L287
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use nectar::{event::TelnetEvent, TelnetCodec};

async fn get_password(frame: Framed<TcpStream, TelnetCodec>) -> Result<String> {
    // Disable echo (eg. hide password input)
    frame.send(TelnetEvent::Will(TelnetOption::Echo)).await?;

    // Handle user input
    let Some(msg) = frame.next().await else {
        frame.send(TelnetEvent::Message("Invalid credentials.".to_string())).await?;
    }

    // Re-enable echo
    frame.send(TelnetEvent::Wont(TelnetOption::Echo)).await?;

    Ok(msg)
}
```

You can check out the [Blossom](https://github.com/robertwayne/blossom) source code for an example of `nectar` in use, in particular the [connection_handler.rs](https://github.com/robertwayne/blossom/blob/dev/blossom/src/connection_handler.rs) file.

## License

nectar source code is dual-licensed under either

- **[MIT License](/docs/LICENSE-MIT)**
- **[Apache License, Version 2.0](/docs/LICENSE-APACHE)**

at your option.
