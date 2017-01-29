extern crate libc;

use std::net;
use std::net::*;
use std::str::FromStr;

lazy_static! {
    pub static ref MACHINE_IDENTIFIER: Option<u32> = machine_identifier();
    pub static ref PROCESS_IDENTIFIER: u8 = process_identifier();
}

const MACHINE_HOSTNAME: &'static str = "localhost";
const LOW_ORDER_TWO_BYTES: i32 = 0x000000ff;

// @FIXME Should clarify the specification of the IP address to be used
fn machine_identifier() -> Option<u32> {
    net::lookup_host(MACHINE_HOSTNAME)
        .ok()
        .and_then(|look_host| {
            look_host.into_iter()
                .find(|addr| addr.is_ipv4())
                .and_then(|addr| Ipv4Addr::from_str(addr.ip().to_string().as_ref()).ok())
                .map(|ipv4addr| u32::from(ipv4addr))
        })
}

// @TODO Implement error handle
fn process_identifier() -> u8 {
    let pid: i32;
    unsafe {
        pid = libc::getpid() as i32;
    }
    (pid & LOW_ORDER_TWO_BYTES) as u8
}



#[cfg(test)]
mod tests {
    use machine;

    #[test]
    fn succ() {
        let r = machine::machine_identifier();
        assert!(r.is_some())
    }
}
