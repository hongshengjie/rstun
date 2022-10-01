use nix::sys::socket;
use nix::sys::time::TimeSpec;
use nix::sys::uio::IoVec;
use std::iter::zip;
use std::net::SocketAddr;
use std::string::String;
use std::thread;
use std::time::Duration;
use stun::message::Message;
use stun::message::*;
use stun::xoraddr::*;

use nix::sys::socket::{
    sockopt, AddressFamily, InetAddr, MsgFlags, RecvMmsgData, SendMmsgData, SockAddr, SockFlag,
    SockType,
};

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
    loop {
        let mut recv_msg_list = std::collections::LinkedList::new();
        let mut receive_buffers = [[0u8; 32]; 1000];
        let iovs: Vec<_> = receive_buffers
            .iter_mut()
            .map(|buf| [IoVec::from_mut_slice(&mut buf[..])])
            .collect();
        for iov in &iovs {
            recv_msg_list.push_back(RecvMmsgData {
                iov,
                cmsg_buffer: None,
            })
        }

        let time_spec = TimeSpec::from_duration(Duration::from_micros(10));
        let resp_result =
            socket::recvmmsg(skt, &mut recv_msg_list, MsgFlags::empty(), Some(time_spec));

        if let Ok(resp) = resp_result {
            let mut msgs = Vec::new();
            let mut src_addr_vec = Vec::new();

            for recv_msg in resp {
                src_addr_vec.push(recv_msg.address)
            }
            for (buf, src_addr_opt) in zip(receive_buffers, src_addr_vec) {
                if let Some(addr) = src_addr_opt {
                    let mut msg = Message::new();
                    msg.raw = buf[..].to_vec();
                    if msg.decode().is_ok() {
                        if msg.typ == BINDING_REQUEST {
                            if let Ok(ad) = addr.to_string().parse::<SocketAddr>() {
                                let xoraddr = XorMappedAddress {
                                    ip: ad.ip(),
                                    port: ad.port(),
                                };
                                msg.typ = BINDING_SUCCESS;
                                msg.write_header();
                                if let Ok(_) = xoraddr.add_to(&mut msg) {
                                    msgs.push((msg.raw, src_addr_opt));
                                }
                            }
                        }
                    }
                }
            }

            let mut send_msg_list = std::collections::LinkedList::new();
            let send_data: Vec<_> = msgs
                .iter()
                .map(|(buf, src_addr)| {
                    let iov = [IoVec::from_slice(&buf[..])];
                    let addr = *src_addr;
                    (iov, addr)
                })
                .collect();

            for (iov, addrx) in send_data {
                let send_msg = SendMmsgData {
                    iov: iov,
                    cmsgs: &[],
                    addr: addrx,
                    _lt: Default::default(),
                };
                send_msg_list.push_back(send_msg);
            }

            _ = socket::sendmmsg(skt, send_msg_list.iter(), MsgFlags::MSG_DONTWAIT);
        }
    }
}
