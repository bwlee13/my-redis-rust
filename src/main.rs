use anyhow::bail;
use std::{
    io::{Read, Write},
    net::SocketAddr,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let stream: std::io::Result<(TcpStream, SocketAddr)> = listener.accept().await;
        match stream {
            Ok((stream, addr)) => {
                println!("accepted new connection on {:?}", addr);
                handle_conn(stream).await?;
            }
            Err(e) => bail!("Failed to accept connection: {}", e),
        }
    }
}

async fn handle_conn(mut stream: TcpStream) -> anyhow::Result<()> {
    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            select! {
                Ok(bytes_read) = stream.read(&mut buffer) => {
                        if bytes_read == 0 {
                            break;
                        }
                        if let Err(e) = stream.write_all(b"+PONG\r\n").await {
                            eprintln!("Failed to write TCP Stream: {}", e);
                            break;
                        }
                    }
                else => {
                    break
                }
            }
        }
    });
    Ok(())
}
