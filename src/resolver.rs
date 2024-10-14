use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
};

use bytes::{BufMut, BytesMut};
use tokio::net::UdpSocket;

use crate::{
    dns_database::DnsDatabase,
    dns_message::{parse_question, DnsHeader, DnsQuestion, ResultCode},
};

pub async fn handle_request(req: BytesMut, addr: SocketAddr, socket: &UdpSocket, db: &DnsDatabase) {
    match build_response(req, &db) {
        Ok(response) => {
            let _ = socket.send_to(&response, &addr).await;
        }
        Err(e) => {
            eprintln!("Failed to build response: {}", e);
        }
    }
}

fn build_response(req: BytesMut, db: &DnsDatabase) -> Result<BytesMut, Box<dyn Error>> {
    if req.len() < 12 {
        return Err("Buffer too small for DNS header".into());
    }

    let mut header = DnsHeader::parse(&req)?;
    let question = parse_question(&req);

    let answer = build_answer(question, db);
    let mut response;
    if answer.is_none() {
        response = header.format(ResultCode::NXDOMAIN, 0)?;
        response.extend_from_slice(&req[12..]);
    } else {
        response = header.format(ResultCode::NOERROR, 1)?;
        response.extend_from_slice(&req[12..]);
        response.put(answer.unwrap());
    }

    Ok(response)
}

fn build_answer(question: DnsQuestion, db: &DnsDatabase) -> Option<BytesMut> {
    let mut res = BytesMut::new();
    let record = db.get_record(question.qname.as_str(), question.qtype);

    if record.is_none() {
        return None;
    }

    let record = record.unwrap();
    res.put_u16(0xc00c);
    let hex_ip: Ipv4Addr = record.value.parse().unwrap();
    res.put_u16(1);
    res.put_u16(1);
    res.put_u32(record.ttl);
    res.put_u16(4);
    res.put_u32(hex_ip.to_bits());
    Some(res)
}
