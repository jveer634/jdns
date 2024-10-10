use bytes::{BufMut, BytesMut};
use std::error::Error;

// A simple representation of a DNS Header
#[derive(Debug)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answers: u16,
    pub authority_rrs: u16,
    pub additional_rrs: u16,
}

#[derive(Debug)]
pub struct DnsQuestion {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl DnsHeader {
    pub fn parse(buf: &BytesMut) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 12 {
            return Err("Buffer too small for DNS header".into());
        }

        let id = (buf[0] as u16) << 8 | buf[1] as u16;
        let flags = (buf[2] as u16) << 8 | buf[3] as u16;
        let questions = (buf[4] as u16) << 8 | buf[5] as u16;
        let answers = (buf[6] as u16) << 8 | buf[7] as u16;
        let authority_rrs = (buf[8] as u16) << 8 | buf[9] as u16;
        let additional_rrs = (buf[10] as u16) << 8 | buf[11] as u16;

        Ok(DnsHeader {
            id,
            flags,
            questions,
            answers,
            authority_rrs,
            additional_rrs,
        })
    }

    pub fn format(&self, buf: &mut BytesMut) {
        buf.put_u16(self.id);
        buf.put_u16(self.flags);
        buf.put_u16(self.questions);
        buf.put_u16(self.answers);
        buf.put_u16(self.authority_rrs);
        buf.put_u16(self.additional_rrs);
    }
}

pub fn parse_question(buf: &[u8]) -> DnsQuestion {
    let mut pos = 12; // Skip the header (first 12 bytes)
    let mut qname = String::new();

    // Read domain name labels
    while buf[pos] != 0 {
        let len = buf[pos] as usize;
        pos += 1;
        for i in 0..len {
            qname.push(buf[pos + i] as char);
        }
        pos += len;
        qname.push('.');
    }
    pos += 1; // Skip the null byte

    let qtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
    let qclass = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);

    DnsQuestion {
        qname,
        qtype,
        qclass,
    }
}
