use std::thread;

use nix::sys::socket::{InetAddr, IpAddr};
use rstun::run_reuse_port_recv_send_mmsg;
fn main() {
    let inet_addr = InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 3478);
    let cpu_num = num_cpus::get();
    let mut i = 1;
    while i <= cpu_num {
        let inet_addr_n = inet_addr.clone();
        thread::spawn(move || run_reuse_port_recv_send_mmsg(inet_addr_n));
        i += 1;
    }
    run_reuse_port_recv_send_mmsg(inet_addr)
}
