#![warn(rust_2018_idioms)]

use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

use argparse::{ArgumentParser, Store, Print};

use futures::FutureExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut listen_addr = String::from("0.0.0.0:12345");
    let mut server_addr = String::from("192.168.50.1:80");
    {
        let version = env!("CARGO_PKG_VERSION");

        let mut ap = ArgumentParser::new();
        ap.set_description("A Simple Tcp Proxy");
        ap.refer(&mut listen_addr)
            .add_option(&["-l", "--listen"], Store, "Specify local address and port");
        ap.refer(&mut server_addr)
            .add_option(&["-s", "--server"], Store, "Specify target address and port");
        ap.add_option(
            &["-v", "--version"],
            Print(format!("tcp-proxy {}", version)),
            "Print the version"
        );
        ap.parse_args_or_exit();
    }
    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let listener = TcpListener::bind(listen_addr.as_str()).await?;

    while let Ok((mut inbound, _)) = listener.accept().await {
        let mut outbound = TcpStream::connect(server_addr.as_str()).await?;

        tokio::spawn(async move {
            copy_bidirectional(&mut inbound, &mut outbound)
                .map(|r| {
                    if let Err(e) = r {
                        println!("Failed to transfer; error={}", e);
                    }
                })
                .await
        });
    }

    Ok(())
}
