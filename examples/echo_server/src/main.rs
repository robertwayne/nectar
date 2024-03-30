use std::{error::Error, net::SocketAddr};

use futures_lite::StreamExt;
use futures_util::sink::SinkExt;
use nectar::{event::TelnetEvent, TelnetCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

async fn handler(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // We construct a 'Frame', which is just a wrapper around the underlying
    // stream that is decoded by the `nectar::TelnetCodec`.
    let mut frame = Framed::new(stream, TelnetCodec::new(1024));

    // Let's send a friendly welcome message to anyone who connects!
    frame
        .send(TelnetEvent::Message(
            "\nWelcome to the nectar telnet server!\nYou can exit by typing \"quit\".\n"
                .to_string(),
        ))
        .await?;

    // In a real application, you would want to handle Some(Err(_)) and None
    // variants, but for this example we'll be succinct for simplicities sake.
    while let Some(Ok(msg)) = frame.next().await {
        match msg {
            TelnetEvent::Message(string) => {
                // We can check for commands...
                if string == "quit" {
                    break;
                }

                // ...or just echo back whatever the user has said!
                frame.send(TelnetEvent::Message(format!("You said: {}\n", string))).await?;
            }
            // We break here to close to connection.
            _ => break,
        }
    }

    // When the above loop breaks we'll send a goodbye message before closing.
    frame.send(TelnetEvent::Message("Goodbye!\n".to_string())).await?;

    Ok(())
}
