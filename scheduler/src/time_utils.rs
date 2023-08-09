use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn timestamp_second(delay_sec: u64) -> u64 {
    if delay_sec == 0 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    } else {
        SystemTime::now()
            .add(Duration::from_secs(delay_sec))
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

pub fn timestamp_ms() -> u128 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
}

pub fn timestamp_micro() -> u128 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros()
}

pub fn timestamp_nanos() -> u128 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos()
}