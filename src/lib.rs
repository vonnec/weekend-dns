use std::net::{Ipv4Addr, Ipv6Addr, IpAddr, UdpSocket};

use domain_name::DomainName;
use record::Kind;

use crate::packet::{Packet, Flags, Question};

pub mod deserialization;
pub mod domain_name;
pub mod packet;
pub mod record;
pub mod serialization;


pub const ROOT_SERVERS: &[(&str, Ipv4Addr, &str, &str)] = &[("a.root-servers.net",Ipv4Addr::new(198,41,0,4),"2001:503:ba3e::2:30","Verisign, Inc."),
("b.root-servers.net", Ipv4Addr::new(199,9,14,201),"001:500:200::b","University of Southern California Information Sciences Institute"),
("c.root-servers.net",Ipv4Addr::new(192,33,4,12),"001:500:2::c","Cogent Communications"),
("d.root-servers.net",Ipv4Addr::new(199,7,91,13),"001:500:2d::d","University of Maryland"),
("e.root-servers.net",Ipv4Addr::new(192,203,230,10),"001:500:a8::e","NASA (Ames Research Center)"),
("f.root-servers.net",Ipv4Addr::new(192,5,5,241),"001:500:2f::f","Internet Systems Consortium, Inc."),
("g.root-servers.net",Ipv4Addr::new(192,112,36,4),"001:500:12::d0d","US Department of Defense (NIC)"),
("h.root-servers.net",Ipv4Addr::new(198,97,190,53),"001:500:1::53","US Army (Research Lab)"),
("i.root-servers.net",Ipv4Addr::new(192,36,148,17),"001:7fe::53","Netnod"),
("j.root-servers.net",Ipv4Addr::new(192,58,128,30),"001:503:c27::2:30","Verisign, Inc."),
("k.root-servers.net",Ipv4Addr::new(193,0,14,129),"001:7fd::1","RIPE NCC"),
("l.root-servers.net",Ipv4Addr::new(199,7,83,42),"001:500:9f::42","ICANN"),
("m.root-servers.net",Ipv4Addr::new(202,12,27,33),"001:dc3::35","WIDE Project")];


pub fn resolve(domain: &str, kind: Kind) -> Option<IpAddr> {

    let Ok(socket) = UdpSocket::bind("0.0.0.0:5353") else {
        println!("failed to bind to port");
        return None;
    };

    {
        let query = Packet::new().with_flags(Flags::new()).with_question(
            Question::new()
                .with_domain_name(&domain)
                .with_kind(kind),
        );
        println!("Sending query: {}", query);
        let buf = query.to_bytes();
        let Ok(_) = socket.send_to(&buf, "198.41.0.4:53") else { // 198.41.0.4:53
            panic!("failed to send packet");
        };
    }
    {
        let mut buf = [0u8; 1024];
        let Ok((count,_addr)) = socket.recv_from(&mut buf) else {
            println!("failed to receive anything");
            return None;
        };
        // println!("debug packet {{");
        // for byte in 0..count.min(1024) {
        //     print!("{:x} ", buf[byte]);
        // }
        // println!("}}");
        let Some(response) = Packet::from_bytes(&buf) else {
            println!("failed to parse packet");
            return None;
        };
        println!("Got response packet: {}", response);
        let answer = response.answers.first().and_then(|r| {
            match r.data {
                record::Content::IPv4(ip) => Some(IpAddr::V4(ip)),
                record::Content::IPv6(ip) => Some(IpAddr::V6(ip)),
                record::Content::DomainName(_) => None,
                record::Content::Text(_) => None,
                record::Content::Other(_) => None,
            }
        });
    answer
    }
    
    
}