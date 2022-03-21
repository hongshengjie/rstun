use clap::{App, Arg};
use nix::sys::socket;
use nix::sys::uio::IoVec;
use std::iter::zip;
use std::net::SocketAddr;
use std::string::String;
use std::thread;
use stun::message::Message;
use stun::message::*;
use stun::xoraddr::*;

use nix::sys::socket::{
    sockopt, AddressFamily, InetAddr, MsgFlags, RecvMmsgData, SendMmsgData, SockAddr, SockFlag,
    SockType,
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
        thread::spawn(move || runmm(addr));
    }
    runmm(svr_addr)
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
    let mut buf = [0u8; 50];
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
        let mut receive_buffers = [[0u8; 50]; 100];
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
        let resdata = socket::recvmmsg(skt, &mut msgs, MsgFlags::empty(), None);
        match resdata {
            Err(_) => continue,
            Ok(res) => {
                let mut address_vec = Vec::new();
                for msg in res {
                    address_vec.push(msg.address)
                }
                for i in 0..address_vec.len() {
                    let buf_opt = receive_buffers.get(i);
                    let addr_opt = address_vec.get(i);
                    match buf_opt {
                        None => continue,
                        Some(buf) => match addr_opt {
                            None => continue,
                            Some(addr) => {
                                let mut req = Message::new();
                                req.raw = buf[..].to_vec();
                                if req.decode().is_ok() {
                                    if req.typ == BINDING_REQUEST {
                                        let addx = *addr;
                                        match addx {
                                            None => continue,
                                            Some(add) => {
                                                if let Ok(ad) =
                                                    add.to_string().parse::<SocketAddr>()
                                                {
                                                    let xoraddr = XorMappedAddress {
                                                        ip: ad.ip(),
                                                        port: ad.port(),
                                                    };
                                                    req.typ = BINDING_SUCCESS;
                                                    req.write_header();
                                                    if let Ok(_) = xoraddr.add_to(&mut req) {
                                                        let _ = socket::sendto(
                                                            skt,
                                                            &req.raw,
                                                            &add,
                                                            MsgFlags::MSG_DONTWAIT,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

fn runmm(r: String) -> std::io::Result<()> {
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
        let mut receive_buffers = [[0u8; 32]; 100];
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
        
        let flag_recv = unsafe{MsgFlags::from_bits_unchecked(0b10000000000000000)};
      
        let resdata = socket::recvmmsg(skt, &mut msgs,flag_recv, None);
        match resdata {
            Err(_) => continue,
            Ok(res) => {
                let mut msgs = Vec::new();
                let mut address_vec = Vec::new();

                for msg in res {
                    address_vec.push(msg.address)
                }
                for (buf, addr_opt) in zip(receive_buffers, address_vec) {
                    match addr_opt {
                        None => continue,
                        Some(addr) => {
                            let mut req = Message::new();
                            req.raw = buf[..].to_vec();
                            if req.decode().is_ok() {
                                if req.typ == BINDING_REQUEST {
                                    if let Ok(ad) = addr.to_string().parse::<SocketAddr>() {
                                        let xoraddr = XorMappedAddress {
                                            ip: ad.ip(),
                                            port: ad.port(),
                                        };
                                        req.typ = BINDING_SUCCESS;
                                        req.write_header();
                                        if let Ok(_) = xoraddr.add_to(&mut req) {
                                            msgs.push((req.raw, addr_opt));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let mut msg_send = std::collections::LinkedList::new();

                let datas: Vec<_> = msgs
                    .iter()
                    .map(|(buf, addr)| {
                        let iov = [IoVec::from_slice(&buf[..])];
                        let add = *addr;
                        (iov, add)
                    })
                    .collect();

                for (iov, addrx) in datas {
                
                    let msg = SendMmsgData {
                        iov: iov,
                        cmsgs: &[],
                        addr: addrx,
                        _lt: Default::default(),
                    };
                    msg_send.push_back(msg);
                }

                let result = socket::sendmmsg(skt, msg_send.iter(), MsgFlags::MSG_DONTWAIT);
                match result {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }
}
