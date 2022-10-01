use std::net::SocketAddr;
use stun::message::*;
use stun::xoraddr::*;
fn main() {
    let src_addr = "127.0.0.1:2211";
    let mut msg = Message::new();
    if let Ok(src_skt_addr) = src_addr.parse::<SocketAddr>() {
        let xoraddr = XorMappedAddress {
            ip: src_skt_addr.ip(),
            port: src_skt_addr.port(),
        };
        msg.typ = BINDING_SUCCESS;
        msg.write_header();
        _ = xoraddr.add_to(&mut msg);
        
        let mut msg2 = Message::new();
        msg2.raw = msg.raw;
        if msg2.decode().is_ok(){
            print!("{}",msg2)
        }
    }
}
