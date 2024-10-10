use std::{error::Error, net::SocketAddr};

use bytes::{BufMut, BytesMut};
use tokio::net::UdpSocket;

use crate::dns_message::{parse_question, DnsHeader};

pub async fn handle_request(req: BytesMut, addr: SocketAddr, socket: &UdpSocket) {
    match build_response(req) {
        Ok(response) => {
            let _ = socket.send_to(&response, &addr).await;
        }
        Err(e) => {
            eprintln!("Failed to build response: {}", e);
        }
    }
}

fn build_response(req: BytesMut) -> Result<BytesMut, Box<dyn Error>> {
    if req.len() < 12 {
        return Err("Buffer too small for DNS header".into());
    }

    let header = DnsHeader::parse(&req)?;
    let question = parse_question(&req);

    println!("Received Question: {}", question.qname);

    // Create a new response buffer
    let mut response = BytesMut::with_capacity(512);

    // Set up the DNS header for the response
    response.put_u16(header.id); // Transaction ID
    response.put_u16(0b10000000); // Flags: QR = 1 (response), Opcode = 0, AA = 1
    response.put_u16(1); // Question Count (1)
    response.put_u16(1); // Answer Count (1)

    // Authority and Additional Record Count (0)
    response.put_u16(0);
    response.put_u16(0);

    // Append the original question section (from the request)
    response.extend_from_slice(&req[12..]); // Append the question part

    // Construct the answer section
    response.put_u16(0xc00c); // Pointer to the domain name in the question
    response.put_u16(0x0001); // Type A (IPv4)
    response.put_u16(0x0001); // Class IN
    response.put_u32(16); // TTL (16 seconds)
    response.put_u16(4); // Length of the IP address (4 bytes)
    response.put_u32(0x7f000001); // 127.0.0.1 in hexadecimal

    // Finalize the response
    println!("Response: {:?}", response);

    Ok(response)
}
