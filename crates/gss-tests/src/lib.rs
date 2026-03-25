pub mod stubs{
    pub mod threshold_stubs;
    pub mod command_stubs;
}

pub mod utils{
    pub mod file_operations;
}

#[cfg(test)]
pub mod tests {
    // Unit Testing
    pub mod unit{
        pub mod settings;
        pub mod thresholds;
        pub mod passwords;
    }
    // Integration Testing
    pub mod api{
        pub mod steam;
        pub mod gog;
        pub mod microsoft_store;
    }
    // Functional Testing
    pub mod functional{
         pub mod commands;
    }
}