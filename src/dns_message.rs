use bytes::{BufMut, BytesMut};
use std::error::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode {
    pub fn from_num(num: u8) -> ResultCode {
        match num {
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            0 | _ => ResultCode::NOERROR,
        }
    }
}

// A simple representation of a DNS Header
#[derive(Debug)]
pub struct DnsHeader {
    pub id: u16,

    pub response: bool, // true for response, false for query
    opcode: u8,
    authoritative_answer: bool,
    truncated_message: bool,
    recursion_desired: bool,
    recursion_available: bool,

    checking_disabled: bool,
    authed_data: bool,
    z: bool,

    result_code: ResultCode,

    pub questions_count: u16,
    pub answers_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum RecordType {
    A = 1,
    CNAME = 5,
    MX = 15,
    TXT = 16,
    AAAA = 28,
}

#[derive(Debug)]
pub struct DnsQuestion {
    pub qname: String,
    pub qtype: RecordType,
    qclass: u16,
}

impl DnsHeader {
    pub fn parse(buf: &BytesMut) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 12 {
            return Err("Buffer too small for DNS header".into());
        }

        let id = (buf[0] as u16) << 8 | buf[1] as u16;
        let flags = (buf[2] as u16) << 8 | buf[3] as u16;
        let questions_count = (buf[4] as u16) << 8 | buf[5] as u16;
        let answers_count = (buf[6] as u16) << 8 | buf[7] as u16;
        let authority_count = (buf[8] as u16) << 8 | buf[9] as u16;
        let additional_count = (buf[10] as u16) << 8 | buf[11] as u16;

        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;

        let recursion_desired = (a & (1 << 0)) > 0;
        let truncated_message = (a & (1 << 1)) > 0;
        let authoritative_answer = (a & (1 << 2)) > 0;
        let opcode = (a >> 3) & 0x0F;
        let response = (a & (1 << 7)) > 0;

        let rescode = ResultCode::from_num(b & 0x0F);
        let checking_disabled = (b & (1 << 4)) > 0;
        let authed_data = (b & (1 << 5)) > 0;
        let z = (b & (1 << 6)) > 0;
        let recursion_available = (b & (1 << 7)) > 0;

        Ok(DnsHeader {
            id,
            response,
            truncated_message,
            result_code: rescode,
            checking_disabled,
            authed_data,
            z,
            authoritative_answer,
            opcode,
            recursion_available,
            recursion_desired,
            questions_count,
            answers_count,
            authority_count,
            additional_count,
        })
    }

    pub fn format(
        &mut self,
        status: ResultCode,
        answers_count: u16,
    ) -> Result<BytesMut, Box<dyn Error>> {
        self.response = true;
        self.result_code = status;
        self.answers_count = answers_count;
        // TODO: handle recursion
        self.recursion_available = false;

        let mut response = BytesMut::with_capacity(512);

        let mut a: u8 = 0;
        let mut b: u8 = 0;

        a |= 1 << 7;
        // Set bits in the 'a' byte
        if self.recursion_desired {
            a |= 1 << 0;
        }
        if self.truncated_message {
            a |= 1 << 1;
        }
        if self.authoritative_answer {
            a |= 1 << 2;
        }
        a |= (self.opcode & 0x0F) << 3; // 4-bit opcode into bits 3-6

        // Set bits in the 'b' byte
        if self.recursion_available {
            b |= 1 << 7;
        }
        b |= (if self.z { 1 } else { 0 }) << 4; // Treat 'z' as a boolean flag
        b |= self.result_code as u8 & 0x0F;

        // Combine a and b into a 16-bit flags field
        let flags: u16 = ((a as u16) << 8) | (b as u16);

        // Set up the DNS header for the response
        response.put_u16(self.id); // Transaction ID
        response.put_u16(flags);
        response.put_u16(self.questions_count); // Question Count (1)
        response.put_u16(self.answers_count); // Answer Count (1)

        response.put_u16(self.authority_count);
        response.put_u16(self.additional_count);

        Ok(response)
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
    let query_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);

    let qtype = match query_type {
        1 => RecordType::A,
        5 => RecordType::CNAME,
        15 => RecordType::MX,
        16 => RecordType::TXT,
        28 => RecordType::AAAA,
        _ => todo!("Implement Error Handling"),
    };

    // for now class is hardcoded
    let qclass = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);

    DnsQuestion {
        qname,
        qtype,
        qclass,
    }
}
