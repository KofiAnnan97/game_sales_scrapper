#[cfg(test)]
pub mod tests {
    pub mod helper;

    // Unit Testing
    pub mod unit{
        pub mod settings;
        pub mod thresholds;
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