use time;
use time::Tm;

pub fn timestamp() -> u64 {
    let current_time = time::get_time();
    let timestamp = (current_time.sec as u64 * 1000) + (current_time.nsec as u64 / 1000 / 1000);
    timestamp
}

pub fn timestamp_with(tm: Tm) -> u64 {
    let current_time = tm.to_timespec();
    let timestamp = (current_time.sec as u64 * 1000) + (current_time.nsec as u64 / 1000 / 1000);
    timestamp
}
