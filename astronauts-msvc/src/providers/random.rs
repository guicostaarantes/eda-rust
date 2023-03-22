use rand::distributions::{Alphanumeric, DistString};

pub struct RandomImpl;

impl RandomImpl {
    pub fn string(size: usize) -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), size)
    }
}
