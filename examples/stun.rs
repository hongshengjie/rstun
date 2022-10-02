use std::net::{IpAddr, Ipv4Addr};
use stun::agent::TransactionId;
use stun::message::*;
use stun::xoraddr::*;
fn main() {
    let msg = create_stun();
    parse_stun(msg.raw)
}
fn create_stun() -> Message {
    let mut msg = Message::new();
    _ = msg.build(&[
        Box::new(TransactionId::default()),
        Box::new(BINDING_REQUEST),
    ]);
    println!("{}", msg);
    msg
}
fn parse_stun(raw: Vec<u8>) {
    let mut resp = Message::new();
    resp.raw = raw;
    if resp.decode().is_ok() {
        resp.typ = BINDING_SUCCESS;
        let xoraddr = XorMappedAddress {
            ip: IpAddr::V4(Ipv4Addr::new(123, 111, 11, 23)),
            port: 2232,
        };
        _ = xoraddr.add_to(&mut resp);
        println!("{}", resp)
    }
}
