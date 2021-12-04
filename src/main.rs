use clap::{App, Arg};
use nix::sys::socket;
use nix::sys::uio::IoVec;
use std::net::SocketAddr;
use std::string::String;
use std::thread;
use stun::message::Message;
use stun::message::*;
use stun::xoraddr::*;

use nix::sys::socket::{
    sockopt, AddressFamily, InetAddr, MsgFlags, RecvMmsgData, RecvMsg, SendMmsgData, SockAddr,
    SockFlag, SockType,
};

fn main() -> std::io::Result<()> {
    let matches = App::new("rstun")
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .takes_value(true)
                .required(true)
                .default_value("3478"),
        )
        .get_matches();

    let port = matches.value_of("PORT").unwrap();
    let svr_addr = format!("0.0.0.0:{}", port);

    let cpus = num_cpus::get();
    let mut index = 1;
    while index <= cpus {
        index = index + 1;
        let addr = String::from(&svr_addr);
        thread::spawn(move || run(addr));
    }
    run(svr_addr)
}

fn run(r: String) -> std::io::Result<()> {
    let address: SocketAddr = r.parse().unwrap();
    let inet = InetAddr::from_std(&address);
    let addr = SockAddr::new_inet(inet);
    let skt = socket::socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .unwrap();
    socket::setsockopt(skt, sockopt::ReusePort, &true).unwrap();
    socket::bind(skt, &addr).unwrap();
    let mut buf = [0u8; 1024];
    loop {
        if let Ok((len, addr_op)) = socket::recvfrom(skt, &mut buf) {
            if let Some(addr) = addr_op {
                {
                    let mut req = Message::new();
                    req.raw = buf[..len].to_vec();
                    if req.decode().is_ok() {
                        if req.typ == BINDING_REQUEST {
                            if let Ok(add) = addr.to_string().parse::<SocketAddr>() {
                                let xoraddr = XorMappedAddress {
                                    ip: add.ip(),
                                    port: add.port(),
                                };
                                req.typ = BINDING_SUCCESS;
                                req.write_header();
                                if let Ok(_) = xoraddr.add_to(&mut req) {
                                    let _ = socket::sendto(
                                        skt,
                                        &req.raw,
                                        &addr,
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

fn runm(r: String) -> std::io::Result<()> {
    let server_address: SocketAddr = r.parse().unwrap();
    let inet = InetAddr::from_std(&server_address);
    let sock_addr = SockAddr::new_inet(inet);
    let skt = socket::socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .unwrap();
    socket::setsockopt(skt, sockopt::ReusePort, &true).unwrap();
    socket::bind(skt, &sock_addr).unwrap();
    //let mut buf = [0u8; 1024];
    loop {
        let mut msgs = std::collections::LinkedList::new();
        let mut receive_buffers = [[0u8; 1024]; 100];
        let iovs: Vec<_> = receive_buffers
            .iter_mut()
            .map(|buf| [IoVec::from_mut_slice(&mut buf[..])])
            .collect();
        for iov in &iovs {
            msgs.push_back(RecvMmsgData {
                iov,
                cmsg_buffer: None,
            })
        }
        let res = socket::recvmmsg(skt, &mut msgs, MsgFlags::MSG_DONTWAIT, None).unwrap();
        let index = 0;

       
        for buf in &receive_buffers[..res.len()] {
            let msg = res[index];
            let mut req = Message::new();
            req.raw = (&buf[..msg.bytes]).to_vec();
            if req.decode().is_ok() {
                if req.typ == BINDING_REQUEST {
                    if let Some(addr) = msg.address {
                        if let Ok(add) = addr.to_string().parse::<SocketAddr>() {
                            let xoraddr = XorMappedAddress {
                                ip: add.ip(),
                                port: add.port(),
                            };
                            req.typ = BINDING_SUCCESS;
                            req.write_header();
                            if let Ok(_) = xoraddr.add_to(&mut req) {
                                let _ =
                                    socket::sendto(skt, &req.raw, &addr, MsgFlags::MSG_DONTWAIT);
                            }
                        }
                    }
                }
            }
        }
    }
}
