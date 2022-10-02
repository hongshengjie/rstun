use nix::sys::socket::{InetAddr, IpAddr};
use rstun::run_single_thread;

fn main() {
    let inet_addr = InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 3478);
    run_single_thread(inet_addr)
}
