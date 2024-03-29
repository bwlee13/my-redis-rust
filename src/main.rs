pub mod redis;

use anyhow::bail;
use anyhow::Result;
use redis::RedisHandler;
use redis::Value;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting TCP Listener...");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream: std::io::Result<(TcpStream, SocketAddr)> = listener.accept().await;

        match stream {
            Ok((stream, addr)) => {
                println!("accepted new connection on {:?}", addr);

                tokio::spawn(async move {
                    handle_conn(stream).await;
                });
            }
            Err(e) => bail!("Failed to accept connection: {}", e),
        }
    }
}

async fn handle_conn(stream: TcpStream) {
    let mut handler = RedisHandler::new(stream);
    println!("Starting read loop");

    loop {
        let value = handler.read_value().await.unwrap();
        println!("Value: {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                _ => panic!("cannot handle command {}", command),
            }
        } else {
            break;
        };

        println!("Response: {:?}", response);

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}
