use std::collections::HashMap;

use crate::dns_message::RecordType;

#[derive(Clone, Debug)]
pub struct DnsRecord {
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32, // Time to live
}

pub struct DnsDatabase {
    records: HashMap<String, Vec<DnsRecord>>,
}

impl DnsDatabase {
    pub fn new() -> Self {
        DnsDatabase {
            records: HashMap::new(),
        }
    }

    pub fn get_record(&self, domain: &str, rtype: RecordType) -> Option<DnsRecord> {
        println!("Search for {:?} {:?} {:#?}", domain, rtype, self.records);
        self.records
            .get(domain)
            .and_then(|records: &Vec<DnsRecord>| records.iter().find(|record| record.record_type == rtype))
            .cloned()
    }

    pub fn add_record(&mut self, domain: &str, record: DnsRecord) {
        self.records
            .entry(domain.to_string())
            .or_default()
            .push(record);
    }

    pub fn add_dummy(&mut self) {
        self.add_record(
            "example.com.",
            DnsRecord {
                record_type: RecordType::A,
                value: "192.0.2.1".to_string(),
                ttl: 3600,
            },
        );
        self.add_record(
            "example.com.",
            DnsRecord {
                record_type: RecordType::A,
                value: "192.0.2.2".to_string(),
                ttl: 3600,
            },
        );

        // Adding AAAA records
        self.add_record(
            "example.com",
            DnsRecord {
                record_type: RecordType::AAAA,
                value: "2001:db8::1".to_string(),
                ttl: 3600,
            },
        );
        self.add_record(
            "example.com",
            DnsRecord {
                record_type: RecordType::AAAA,
                value: "2001:db8::2".to_string(),
                ttl: 3600,
            },
        );

        // Adding CNAME records
        self.add_record(
            "www.example.com",
            DnsRecord {
                record_type: RecordType::CNAME,
                value: "example.com".to_string(),
                ttl: 3600,
            },
        );
        self.add_record(
            "blog.example.com",
            DnsRecord {
                record_type: RecordType::CNAME,
                value: "example.com".to_string(),
                ttl: 3600,
            },
        );

        // Adding MX records
        self.add_record(
            "example.com",
            DnsRecord {
                record_type: RecordType::MX,
                value: "mail.example.com".to_string(),
                ttl: 3600,
            },
        );
        self.add_record(
            "example.com",
            DnsRecord {
                record_type: RecordType::MX,
                value: "altmail.example.com".to_string(),
                ttl: 3600,
            },
        );
    }
}
