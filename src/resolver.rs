use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use bytes::{BufMut, BytesMut};
use tokio::net::UdpSocket;

use crate::{
    dns_database::DnsDatabase,
    dns_message::{parse_question, DnsHeader, DnsQuestion, RecordType, ResultCode},
};

pub async fn handle_request(req: BytesMut, addr: SocketAddr, socket: &UdpSocket, db: &DnsDatabase) {
    let response = build_response(req, &db);
    let _ = socket.send_to(&response, &addr).await;
}

fn build_response(req: BytesMut, db: &DnsDatabase) -> BytesMut {
    let mut header = DnsHeader::parse(&req);
    if header.result_code == ResultCode::FORMERR {
        return header.format(ResultCode::FORMERR, 0);
    }

    let question = match parse_question(&req) {
        Ok(x) => x,
        Err(_) => {
            return header.format(ResultCode::FORMERR, 0);
        }
    };

    let answer = build_answer(question, db);
    // println!("Answer {:#?}", answer);
    let mut response;
    if answer.is_none() {
        response = header.format(ResultCode::NXDOMAIN, 0);
        response.extend_from_slice(&req[12..]);
    } else {
        response = header.format(ResultCode::NOERROR, 1);
        response.extend_from_slice(&req[12..]);
        response.put(answer.unwrap());
    }

    response
}

fn build_answer(question: DnsQuestion, db: &DnsDatabase) -> Option<BytesMut> {
    let mut res = BytesMut::new();
    let record = db.get_record(question.qname.trim_end_matches('.'), question.qtype);

    // println!("Record {:?}", record);

    if record.is_none() {
        return None;
    }

    let record = record.unwrap();
    res.put_u16(0xc00c);
    match record.record_type {
        RecordType::A => {
            let ip: Ipv4Addr = record.value.parse().unwrap();
            res.put_u16(1);
            res.put_u16(1);
            res.put_u32(record.ttl);
            res.put_u16(4);
            res.put_u32(ip.to_bits());
        }

        RecordType::AAAA => {
            let ip: Ipv6Addr = record.value.parse().unwrap();
            res.put_u16(28);
            res.put_u16(1);
            res.put_u32(record.ttl);
            res.put_u16(16);
            res.put_u128(ip.into());
        }

        RecordType::MX => {
            let encoded_domain: Vec<u8> = encode_domain_name(&record.value);
            let default_preference: u16 = 10;
            res.put_u16(15);
            res.put_u16(1);
            res.put_u32(record.ttl);
            let rdata_len = 2 + encoded_domain.len(); // 2 bytes for preference + length of the encoded domain name
            res.put_u16(rdata_len as u16);
            res.put_u16(default_preference);
            res.put_slice(&encoded_domain);
        }

        RecordType::CNAME => {
            let encoded_domain: Vec<u8> = encode_domain_name(&record.value);
            res.put_u16(5);
            res.put_u16(1);
            res.put_u32(record.ttl);
            let rdata_len = 2 + encoded_domain.len(); // 2 bytes for preference + length of the encoded domain name
            res.put_u16(rdata_len as u16);
            res.put_slice(&encoded_domain);
        }

        RecordType::TXT => {
            let encoded_domain: Vec<u8> = encode_domain_name(&record.value);
            res.put_u16(16);
            res.put_u16(1);
            res.put_u32(record.ttl);
            let rdata_len = 2 + encoded_domain.len(); // 2 bytes for preference + length of the encoded domain name
            res.put_u16(rdata_len as u16);
            res.put_slice(&encoded_domain);
        }
    }
    Some(res)
}

fn encode_domain_name(domain: &str) -> Vec<u8> {
    let mut encoded = Vec::new();

    for label in domain.split('.') {
        let label_len = label.len();
        encoded.push(label_len as u8); // Prefix with the length of the label
        encoded.extend_from_slice(label.as_bytes()); // Add the label itself
    }

    encoded.push(0);
    encoded
}
