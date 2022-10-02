use std::net::SocketAddr;
use stun::message::*;
use stun::xoraddr::*;
use nix::sys::socket::SockAddr;

pub fn process_stun_request(src_addr: SockAddr, buf: Vec<u8>) -> Option<Message> {
    let mut msg = Message::new();
    msg.raw = buf;
    if msg.decode().is_err() {
        return None;
    }
    if msg.typ != BINDING_REQUEST {
        return None;
    }
    match src_addr.to_string().parse::<SocketAddr>() {
        Err(_) => return None,
        Ok(src_skt_addr) => {
            let xoraddr = XorMappedAddress {
                ip: src_skt_addr.ip(),
                port: src_skt_addr.port(),
            };
            msg.typ = BINDING_SUCCESS;
            msg.write_header();
            match xoraddr.add_to(&mut msg) {
                Err(_) => None,
                Ok(_) => Some(msg),
            }
        }
    }
}
