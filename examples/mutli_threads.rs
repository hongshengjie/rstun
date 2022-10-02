use nix::sys::socket;
use rstun::process_stun_request;
use nix::sys::socket::{
    sockopt, AddressFamily, InetAddr, IpAddr, MsgFlags, SockAddr, SockFlag, SockType,
};
use std::thread;

fn main() {
    let inet_addr = InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 3478);
    let cpu_num = num_cpus::get();
    let mut i = 1;
    while i <= cpu_num {
        let inet_addr_n = inet_addr.clone();
        thread::spawn(move || run(inet_addr_n));
        i += 1;
    }
    run(inet_addr)
}

fn run(inet_addr: InetAddr) {
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
        match socket::recvfrom(skt, &mut buf) {
            Err(_) => continue,
            Ok((len, src_addr_op)) => match src_addr_op {
                None => {}
                Some(src_addr) => {
                    if let Some(msg) = process_stun_request(src_addr, buf[..len].to_vec()) {
                        _ = socket::sendto(skt, &msg.raw, &src_addr, MsgFlags::MSG_DONTWAIT);
                    }
                }
            },
        }
    }
}
