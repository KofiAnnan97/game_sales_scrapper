#[cfg(test)]
pub mod tests {
    pub mod unit{
        pub mod settings;
        pub mod thresholds;
    }
    pub mod api{
        pub mod steam;
        pub mod gog;
        pub mod microsoft_store;
    }
    pub mod functional{
         pub mod commands;
    }
}