use std::{error::Error, net::SocketAddr};

use bytes::{Buf, BufMut, BytesMut};
use tokio::net::UdpSocket;

use crate::dns_message::{parse_question, DnsHeader, ResultCode};

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

    let mut header = DnsHeader::parse(&req)?;
    println!("Header: {:#?}", header);

    let question = parse_question(&req);

    println!("Question: {:#?}", question);

    // Create a new response buffer

    let mut response = header.format(ResultCode::NOERROR, 1)?;

    // Append the original question section (from the request)
    response.extend_from_slice(&req[12..]); // Append the question part

    // Construct the answer section
    response.put_u16(0xc00c); // Pointer to the domain name in the question
    response.put_u16(0x0001); // Type A (IPv4)
    response.put_u16(0x0001); // Class IN
    response.put_u32(16); // TTL (16 seconds)
    response.put_u16(4); // Length of the IP address (4 bytes)
    response.put_u32(0x7f000001); // 127.0.0.1 in hexadecimal

    Ok(response)
}
