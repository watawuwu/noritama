use byteorder::{ByteOrder, BigEndian};
use std::fmt;

lazy_static! {
    pub static ref MAX_INTERNAL_TIMESTAMP: u64 = 2u64.pow(40) - 1u64;
}

#[derive(Debug)]
pub struct Flake {
    pub internal_timestamp: u64,
    pub sequence: u16,
    pub machine_identifier: u32,
    pub process_identifier: u8,
    pub start_epoch: u64,
}

impl fmt::Display for Flake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "hex: {}({:?}), unix_ts: {}, internal_ts: {}, seq: {}, mid: {}, pid: {}, start_epo: {}",
               self.hex(),
               self.octets(),
               self.unix_timestamp(),
               self.internal_timestamp,
               self.sequence,
               self.machine_identifier,
               self.process_identifier,
               self.start_epoch)
    }
}

pub trait Flakable {
    fn hex(&self) -> String;
    fn octets(&self) -> [u8; 12];
    fn unix_timestamp(&self) -> u64;
}

impl Flakable for Flake {
    fn hex(&self) -> String {
        let buf = self.octets();
        let left = BigEndian::read_u32(&buf[0..4]);
        let mid = BigEndian::read_u32(&buf[4..8]);
        let right = BigEndian::read_u32(&buf[8..12]);

        format!("{0:>08x}{1:>08x}{2:>08x}", left, mid, right)
    }

    // big endian
    fn octets(&self) -> [u8; 12] {
        [(self.internal_timestamp >> 32) as u8,
         (self.internal_timestamp >> 24) as u8,
         (self.internal_timestamp >> 16) as u8,
         (self.internal_timestamp >> 8) as u8,
         self.internal_timestamp as u8,
         (self.sequence >> 8) as u8,
         self.sequence as u8,
         (self.machine_identifier >> 24) as u8,
         (self.machine_identifier >> 16) as u8,
         (self.machine_identifier >> 8) as u8,
         self.machine_identifier as u8,
         self.process_identifier]
    }

    fn unix_timestamp(&self) -> u64 {
        self.internal_timestamp + self.start_epoch
    }
}


#[cfg(test)]
mod tests {

    use flake;
    use flake::*;
    use regex::Regex;

    use rustc_serialize::hex::FromHex;

    #[test]
    fn suc() {
        let id_regex: &'static str = r"^[0-9a-z]{24}$";

        let internal_timestamp = 27691315937u64;
        let sequence = 0u16;
        let machine_identifier = 2130706433u32;
        let process_identifier = 138u8;
        let start_epoch = 1457913600000u64;

        let a = flake::Flake {
            internal_timestamp: internal_timestamp,
            sequence: sequence,
            machine_identifier: machine_identifier,
            process_identifier: process_identifier,
            start_epoch: start_epoch,
        };

        let re = Regex::new(id_regex).unwrap();

        assert!(re.is_match(&a.hex().as_str()));
        assert!((a.hex().as_ref() as &str).from_hex().unwrap() == a.octets());
        assert!(a.unix_timestamp() == internal_timestamp + start_epoch);
    }

}
