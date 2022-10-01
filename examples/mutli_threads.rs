use nix::sys::socket;
use std::net::SocketAddr;
use std::string::String;
use std::thread;
use stun::message::*;
use stun::xoraddr::*;

use nix::sys::socket::{sockopt, AddressFamily, InetAddr, MsgFlags, SockAddr, SockFlag, SockType};

fn main() {
    let addr_str = format!("0.0.0.0:{}", 3478);
    let cpus = num_cpus::get();
    let mut i = 1;
    while i <= cpus {
        i += 1;
        let addr_str2 = String::from(&addr_str);
        thread::spawn(move || run(addr_str2));
    }
    run(addr_str)
}

fn run(addr_str: String) {
    let socket_addr: SocketAddr = addr_str.parse().unwrap();
    let inet_addr = InetAddr::from_std(&socket_addr);
    let skt_addr = SockAddr::new_inet(inet_addr);
    let skt = socket::socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .unwrap();
    socket::setsockopt(skt, sockopt::ReusePort, &true).unwrap();
    socket::bind(skt, &skt_addr).unwrap();
    let mut buf = [0u8; 50];
    loop {
        if let Ok((len, src_addr_op)) = socket::recvfrom(skt, &mut buf) {
            if let Some(src_addr) = src_addr_op {
                {
                    let mut msg = Message::new();
                    msg.raw = buf[..len].to_vec();
                    if msg.decode().is_ok() {
                        if msg.typ == BINDING_REQUEST {
                            if let Ok(src_skt_addr) = src_addr.to_string().parse::<SocketAddr>() {
                                let xoraddr = XorMappedAddress {
                                    ip: src_skt_addr.ip(),
                                    port: src_skt_addr.port(),
                                };
                                msg.typ = BINDING_SUCCESS;
                                msg.write_header();
                                if let Ok(_) = xoraddr.add_to(&mut msg) {
                                    _ = socket::sendto(
                                        skt,
                                        &msg.raw,
                                        &src_addr,
                                        MsgFlags::MSG_DONTWAIT,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
