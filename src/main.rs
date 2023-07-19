use std::env;
use std::net::{UdpSocket, Ipv4Addr};

use weekend_dns::{packet::*, resolve};
use weekend_dns::record::Kind;
use weekend_dns::ROOT_SERVERS;

const ROOT_SERVER: Ipv4Addr = ROOT_SERVERS[0].1;

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let domain_str = args.next().unwrap_or("www.google.com".to_string());

    let record_kind: Kind = args
        .next()
        .and_then(|s| s.parse::<u16>().ok().and_then(|n| n.try_into().ok()))
        .unwrap_or(Kind::A);

    println!("requesting address for {}", domain_str);
    let address = resolve(&domain_str, record_kind);
    println!("got {:?}", address);
}
