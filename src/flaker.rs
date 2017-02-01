use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::io::prelude::*;

use byteorder::{ByteOrder, BigEndian};
use util::{time as utime, bread};
use rustc_serialize::hex::FromHex;
use time::Tm;

use machine;
use flake;
use flake::Flake;
use error::Error;

pub trait Flakerable {
    // @see 2016-03-14
    const DEFAULT_START_EPOC: u64 = 1457913600000u64;
    const TIMESTAMP_MASK: u64 = 0xffffffffffff0000u64;
    const SEQUENCE_MASK: u64 = 0x000000000000ffffu64;
    const SEQUENCE_BIT: u8 = 16u8;

    fn id(&self) -> Result<Flake, Error>;
    fn id_with_str(&self, id: &str) -> Result<Flake, Error>;
    fn id_with_bytes(&self, id: &[u8]) -> Result<Flake, Error>;
    fn unix_timestamp_from(&self, id: &str) -> Result<u64, Error>;
    fn unix_ts2ts_seq(&self, timestamp: &u64) -> u64;
    fn timestamp_with_sequence(&self) -> (u64, u16);
    fn timestamp(&self) -> u64;
    fn eq_timestamp(&self, t1: &u64, t2: &u64) -> bool;
    fn inc_timestamp_sequence(&self, now: &u64, last: &u64) -> u64;
    fn timeseq(&self) -> u64;

    fn timestamp_from(id: &str) -> Result<u64, Error> {
        let bytes = Self::parse_hex_str(id)?;
        let mut bytes = &bytes[..];
        let mut timestamp = [0u8; 5];
        bytes.read_exact(&mut timestamp)
            .map_err(|_| Error::IOError(format!("Failed to ensure the timestamp. id: {}", id)))?;
        Ok(BigEndian::read_u64(&Self::pad_timestamp(&timestamp)))
    }

    fn timestamp_of(time_sequence: &u64) -> u64 {
        (time_sequence & Self::TIMESTAMP_MASK) >> Self::SEQUENCE_BIT
    }
    fn sequence_of(time_sequence: &u64) -> u16 {
        (time_sequence & Self::SEQUENCE_MASK) as u16
    }

    fn parse_hex_str(s: &str) -> Result<[u8; 12], Error> {
        let b = s.from_hex()
            .map_err(|_| Error::ParseError(format!("Not hex string. in:{}", s)))?;

        let mut buf = [0; 12];
        (&b[..]).read_exact(&mut buf)
            .map_err(|_| Error::ParseError(format!("Character length is insufficient. in:{}", s)))?;
        Ok(buf)
    }

    fn pad_timestamp(t: &[u8; 5]) -> [u8; 8] {
        // @FIXME Is not a good method
        [0u8, 0u8, 0u8, t[0], t[1], t[2], t[3], t[4]]
    }
}

#[derive(Debug)]
pub struct Flaker {
    start_epoch: u64,
    timestamp_correction: i64,
    last_timestamp: AtomicUsize,
}

impl Flakerable for Flaker {
    fn id(&self) -> Result<Flake, Error> {
        let (ts, seq) = self.timestamp_with_sequence();
        let machine_identifier =
            (*machine::MACHINE_IDENTIFIER)
                .ok_or(Error::InitError(format!("Failed to ensure machine identifier")))?;
        let flake = Flake {
            internal_timestamp: ts,
            sequence: seq,
            machine_identifier: machine_identifier,
            process_identifier: *machine::PROCESS_IDENTIFIER,
            start_epoch: self.start_epoch,
        };

        debug!("{:?}", flake);

        Ok(flake)
    }

    fn id_with_str(&self, id: &str) -> Result<Flake, Error> {
        let s = Self::parse_hex_str(id)?;
        self.id_with_bytes(&s)
    }

    fn id_with_bytes(&self, id: &[u8]) -> Result<Flake, Error> {
        let mut idm = id;
        let mut timestamp = [0u8; 5];
        let mut sequence = [0u8; 2];
        let mut machine_identifier = [0u8; 4];
        let mut process_identifier = [0u8; 1];

        idm.read_exact(&mut timestamp)
            .map_err(|_| Error::IOError(format!("Failed to ensure the timestamp. id: {:?}", id)))?;
        idm.read_exact(&mut sequence)
            .map_err(|_| Error::IOError(format!("Failed to ensure the sequence. id: {:?}", id)))?;
        idm.read_exact(&mut machine_identifier)
            .map_err(|_| {
                Error::IOError(format!("Failed to ensure the machine_identifier. id: {:?}", id))
            })?;
        idm.read_exact(&mut process_identifier)
            .map_err(|_| {
                Error::IOError(format!("Failed to ensure the process_identifier. id: {:?}", id))
            })?;

        let ts = &Self::pad_timestamp(&timestamp);

        let r =
            Flake {
                internal_timestamp: bread::read_u64(ts).map_err(|_| {
                        Error::IOError(format!("Failed to convert the timestamp. {:?}", ts))
                    })?,
                sequence: bread::read_u16(&sequence).map_err(|_| {
                        Error::IOError(format!("Failed to convert the sequence. {:?}", sequence))
                    })?,
                machine_identifier: bread::read_u32(&machine_identifier).map_err(|_| {
                        Error::IOError(format!("Failed to convert the machine_identifier. {:?}",
                                               machine_identifier))
                    })?,
                process_identifier: process_identifier[0], // @FIXME panic???
                start_epoch: self.start_epoch,
            };

        Ok(r)
    }

    fn timestamp(&self) -> u64 {
        (utime::timestamp() as i64 + self.timestamp_correction) as u64
    }

    fn eq_timestamp(&self, t1: &u64, t2: &u64) -> bool {
        Flaker::timestamp_of(t1) == Flaker::timestamp_of(t2)
    }

    // @FIXME Return ref
    fn inc_timestamp_sequence(&self, now: &u64, last: &u64) -> u64 {
        if last >= now {
            let candidate = last + 1;
            if self.eq_timestamp(&candidate, &last) {
                candidate
            } else {
                *now
            }
        } else {
            *now
        }
    }


    fn timeseq(&self) -> u64 {
        let now = self.unix_ts2ts_seq(&self.timestamp());
        let last = self.last_timestamp.load(Ordering::Relaxed) as u64;
        let update = self.inc_timestamp_sequence(&now, &last);

        match self.last_timestamp.compare_exchange(last as usize,
                                                   update as usize,
                                                   Ordering::Acquire,
                                                   Ordering::Relaxed) {
            Ok(_) => update,
            Err(_) => self.timeseq(),
        }

    }

    fn timestamp_with_sequence(&self) -> (u64, u16) {
        let time_sequence = self.timeseq();
        (Self::timestamp_of(&time_sequence), Self::sequence_of(&time_sequence))
    }

    fn unix_timestamp_from(&self, id: &str) -> Result<u64, Error> {
        Self::timestamp_from(id).map(|t| t + self.start_epoch)
    }

    fn unix_ts2ts_seq(&self, timestamp: &u64) -> u64 {
        (timestamp - self.start_epoch) << Self::SEQUENCE_BIT
    }
}


impl Flaker {
    pub fn new() -> Result<Flaker, Error> {
        Self::new_with_epoch_timestamp(Self::DEFAULT_START_EPOC, None)
    }

    pub fn new_with_epoch_tm(start_tm: Tm) -> Result<Flaker, Error> {
        let star_epoch = utime::timestamp_with(start_tm);
        Self::new_with_epoch_timestamp(star_epoch, None)
    }

    pub fn new_with_epoch_timestamp(start_epoch: u64,
    maybe_timestamp_correction: Option<i64>)
                                    -> Result<Flaker, Error> {
        let ts_correction = maybe_timestamp_correction.unwrap_or(0i64);

        if !Self::valid_start_epoch(&start_epoch, &ts_correction) {
            return Err(Error::InvalidArgError(format!("Invalid value of start_epoch. {}",
                                                      start_epoch)
                .to_string()));
        }

        let f = Flaker {
            start_epoch: start_epoch,
            timestamp_correction: ts_correction,
            last_timestamp: ATOMIC_USIZE_INIT,
        };

        debug!("{:?}", f);

        Ok(f)
    }

    fn valid_start_epoch(start_epoch: &u64, timestamp_correction: &i64) -> bool {
        let now = utime::timestamp();


        // @TODO
        if timestamp_correction.is_negative() && timestamp_correction.abs() as u64 > now {
            warn!("Invalid timestamp_correction. timestamp_correction: {}, now: {}",
                  &timestamp_correction,
                  &now);
            return false;
        }

        // @TODO
        let virtual_now = (now as i64 + timestamp_correction) as u64;

        if start_epoch > &virtual_now {
            warn!("Invalid start_epoch. start_epoch: {}, virtual_now: {}",
                  &start_epoch,
                  &virtual_now);

            return false;
        }


        let internal_timestamp = virtual_now - start_epoch;

        if internal_timestamp > *flake::MAX_INTERNAL_TIMESTAMP {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::sync::mpsc;

    use time;

    use flake;
    use flake::{Flakable, Flake};
    use flaker::{Flakerable, Flaker};
    use util::time as utime;

    fn flaker() -> Flaker {
        Flaker::new().unwrap()
    }

    #[test]
    fn publish() {
        let start = utime::timestamp();
        let c: Flaker = flaker();
        let a: Flake = c.id().unwrap();
        let end = utime::timestamp();

        assert!(a.internal_timestamp != 0);
        assert!(a.internal_timestamp != a.unix_timestamp());
        assert!(a.machine_identifier != 0u32);

        assert!(a.unix_timestamp() >= start);
        assert!(a.unix_timestamp() <= end);
    }

    #[test]
    fn decode() {
        let e_id = "0652115c9c00007f000001ce";
        let e_its = 27146673308u64;
        let e_uts = 1485060273308u64;
        let e_seq = 0u16;
        let e_mid = 2130706433u32;
        let e_pid = 206u8;

        let a: Flake = flaker().id_with_str(e_id).unwrap();

        assert!(a.hex().as_str() == e_id);
        assert!(a.unix_timestamp() == e_uts);
        assert!(a.internal_timestamp == e_its);
        assert!(a.sequence == e_seq);
        assert!(a.machine_identifier == e_mid);
        assert!(a.process_identifier == e_pid);
    }

    #[test]
    fn timestamp_format() {
        let e_id = "0652115c9c00007f000001ce";
        let e_tm = "2017-01-22T04:44:33Z";
        let a: Flake = flaker().id_with_str(&e_id).unwrap();

        let sec = a.unix_timestamp() / 1000;
        let tm = time::strptime(sec.to_string().as_str(), "%s").unwrap();

        assert!(tm.rfc3339().to_string().as_str() == e_tm);
    }

    #[test]
    fn of() {
        let a: Flake = flaker().id().unwrap();
        let a_id = &a.hex();

        assert!(a.internal_timestamp == Flaker::timestamp_from(&a_id).unwrap());
    }

    #[test]
    #[quickcheck]
    #[allow(unused_attributes)]
    fn invalid_id_str(i: String) {
        let a = flaker().id_with_str(i.as_str());
        assert!(a.is_err());
    }

    #[test]
    fn invalid_id_bytes() {
        let a = flaker().id_with_bytes(&[0; 3]);
        assert!(a.is_err());
    }

    fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
        let mut rev = vec![];
        for x in xs {
            rev.insert(0, x.clone())
        }
        rev
    }

    fn ids(c: &Flaker, i: u32) -> Vec<String> {
        (0..i)
            .map(|_| c.id().unwrap().hex())
            .collect()
    }

    #[test]
    fn k_orderd() {
        let a = ids(&flaker(), 100);
        assert!(a == reverse(&reverse(&a)));
    }

    #[test]
    fn unique() {
        let count = 1000;
        let thread_num = 10;

        let c: Arc<Flaker> = Arc::new(flaker());

        let (tx, rx) = mpsc::channel();
        for _ in 0..thread_num {
            let m_tx = tx.clone();
            let c = c.clone();
            thread::spawn(move || m_tx.send(ids(&c, count)).unwrap());
        }

        drop(tx);
        let mut t = Vec::new();
        for r in rx.iter() {
            for l in r {
                t.push(l);
            }
        }

        let a_len = t.len();
        t.dedup();
        let e_len = t.len();

        assert!(a_len == e_len);
        assert!(a_len == (count * thread_num) as usize);
    }

    #[test]
    fn correction() {
        let timestamp_correction: i64 = 3600 * 24 * 365;
        let time_adjustment: u64 = 10000;
        let c: Flaker = Flaker::new_with_epoch_timestamp(Flaker::DEFAULT_START_EPOC,
                                                         Some(timestamp_correction))
            .unwrap();

        let id = c.id().unwrap();

        assert!(c.start_epoch == Flaker::DEFAULT_START_EPOC);
        assert!(c.timestamp_correction == timestamp_correction);

        assert!(id.start_epoch == Flaker::DEFAULT_START_EPOC);
        assert!(id.unix_timestamp() > utime::timestamp());
        assert!(id.unix_timestamp() < utime::timestamp() + timestamp_correction as u64 + time_adjustment);
    }

    #[test]
    fn start_epoch() {
        let tm = time::strptime("1983-03-14", "%Y-%m-%d").unwrap();
        let start_epoch = utime::timestamp_with(tm);
        let time_adjustment = 1000000;
        let now = utime::timestamp();

        // Minimum internal timestamp
        let minimum_timestamp_correction = -((now - time_adjustment - start_epoch) as i64);
        let a1 = Flaker::new_with_epoch_timestamp(start_epoch, Some(minimum_timestamp_correction));
        assert!(a1.is_ok());

        // Over minimum internal timestamp
        let over_minimum_timestamp_correction = -((now + time_adjustment - start_epoch) as i64);
        let a2 = Flaker::new_with_epoch_timestamp(start_epoch,
                                                  Some(over_minimum_timestamp_correction));
        assert!(a2.is_err());

        // Maximum internal timestamp
        let maximum_timestamp_correction =
            ((*flake::MAX_INTERNAL_TIMESTAMP - time_adjustment) - (now - start_epoch)) as i64;
        let a3 = Flaker::new_with_epoch_timestamp(start_epoch, Some(maximum_timestamp_correction));
        assert!(a3.is_ok());

        // Over maximum internal timestamp
        let over_maximum_timestamp_correction =
            ((*flake::MAX_INTERNAL_TIMESTAMP + time_adjustment) - (now - start_epoch)) as i64;
        let a4 = Flaker::new_with_epoch_timestamp(start_epoch,
                                                  Some(over_maximum_timestamp_correction));
        assert!(a4.is_err());
    }
}
