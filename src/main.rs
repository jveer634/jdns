mod dns_message;
mod resolver;
mod udp;

use simple_logger::SimpleLogger;
use udp::UdpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init().unwrap();

    let server = UdpServer::new("127.0.0.1:5300").await?;
    println!("DNS Server started at port 5300");
    server.run().await
}
