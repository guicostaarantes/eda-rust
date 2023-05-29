use uuid::Uuid;

pub struct RandomImpl;

impl RandomImpl {
    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }
}
