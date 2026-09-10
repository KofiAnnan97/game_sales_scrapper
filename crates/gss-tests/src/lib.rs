pub mod stubs {
    pub mod command_stubs;
    pub mod threshold_stubs;
}

pub mod utils {
    pub mod file_operations;
    pub mod tmp_setup;
}

#[cfg(test)]
pub mod tests {
    // Unit Testing
    pub mod unit {
        pub mod algorithms;
        pub mod env_vars;
        pub mod passwords;
        pub mod properties;
        pub mod settings;
        pub mod thresholds;
    }
    // Integration Testing
    pub mod api {
        pub mod gog;
        pub mod microsoft_store;
        pub mod steam;
    }
    // Functional Testing
    pub mod functional {
        pub mod commands;
    }
}
