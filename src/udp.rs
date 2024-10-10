use crate::resolver::handle_request;
use bytes::BytesMut;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct UdpServer {
    socket: Arc<UdpSocket>,
}

impl UdpServer {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(UdpServer {
            socket: Arc::new(socket),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0u8; 512]; // Create a fixed-size buffer
        loop {
            let (size, src) = match self.socket.recv_from(&mut buf).await {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Failed to receive data: {}", err);
                    continue; // Skip to the next iteration
                }
            };
            if size == 0 {
                println!("Received an empty request from {}", src);
                continue; // Skip processing this request
            }

            println!("Received {} bytes from {:?}", size, src);
            println!("Raw request bytes: {:?}", &buf[..size]); // Log the raw bytes received

            // Ensure the buffer has the expected data before processing
            if buf.len() < 12 {
                println!("Received buffer too small for DNS header");
                continue;
            }

            handle_request(BytesMut::from(&buf[..size]), src, &self.socket).await;
        }
    }
}
