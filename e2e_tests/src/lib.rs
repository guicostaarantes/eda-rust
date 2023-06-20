#[cfg(test)]
mod test_astronauts;
#[cfg(test)]
mod test_missions;
#[cfg(test)]
mod test_time_elapsed;

#[cfg(test)]
mod utils {
    pub(crate) fn auth_url(path: &str) -> String {
        let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:7001".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn astronauts_url(path: &str) -> String {
        let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:7002".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn missions_url(path: &str) -> String {
        let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:7003".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn current_timestamp() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected timestamp to be calculated")
            .as_millis()
    }
}
