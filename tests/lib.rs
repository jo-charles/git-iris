use git_iris::logger;

// Common test utilities shared across test modules
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    fn _setup() {
        let _ = logger::init(); // Initialize the logger
        logger::enable_logging(); // Enable logging
        logger::set_log_to_stdout(true); // Set logging to stdout
    }
}
