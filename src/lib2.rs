use std::net::SocketAddr;
use stun::message::*;
use stun::xoraddr::*;

use nix::sys::socket::{
    self, sockopt, AddressFamily, InetAddr, IpAddr, MsgFlags, RecvMmsgData, SendMmsgData, SockAddr,
    SockFlag, SockType,
};
use nix::sys::time::TimeSpec;
use nix::sys::uio::IoVec;

use rstun::process_stun_request;
use std::iter::zip;
use std::time::Duration;

pub fn run_reuse_port_recv_send_mmsg(inet_addr: InetAddr) {
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
        let mut receive_buffers = [[0u8; 50]; 1000];
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

        match resp_result {
            Err(_) => {}
            Ok(resp) => {
                let mut msgs = Vec::new();
                let mut src_addr_vec = Vec::new();

                for recv_msg in resp {
                    src_addr_vec.push(recv_msg.address)
                }
                for (buf, src_addr_opt) in zip(receive_buffers, src_addr_vec) {
                    match src_addr_opt {
                        None => {}
                        Some(src_addr) => {
                            if let Some(msg) = process_stun_request(src_addr, buf.to_vec()) {
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
                        iov,
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
}
