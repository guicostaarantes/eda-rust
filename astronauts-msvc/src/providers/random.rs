use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use uuid::Uuid;

pub struct RandomImpl;

impl RandomImpl {
    pub fn string(size: usize) -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), size)
    }
}

impl RandomImpl {
    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }
}
