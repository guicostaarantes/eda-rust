#[cfg(test)]
mod test_astronauts;
#[cfg(test)]
mod test_missions;
#[cfg(test)]
mod test_time_elapsed;

#[cfg(test)]
mod utils {
    pub(crate) fn auth_url(path: &str) -> String {
        let base_url =
            std::env::var("AUTH_BASE_URL").unwrap_or("http://localhost:30001".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn astronauts_url(path: &str) -> String {
        let base_url =
            std::env::var("ASTRONAUTS_BASE_URL").unwrap_or("http://localhost:30002".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn missions_url(path: &str) -> String {
        let base_url =
            std::env::var("MISSIONS_BASE_URL").unwrap_or("http://localhost:30003".to_string());
        format!("{}{}", base_url, path)
    }

    pub(crate) fn current_timestamp() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected timestamp to be calculated")
            .as_millis()
    }
}
