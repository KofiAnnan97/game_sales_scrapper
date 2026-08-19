#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOptions {
    Steam,
    GOG,
    MicrosoftStore,
    All
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceOptions {
    None,
    Under5,
    Under10,
    Under25,
    Custom
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOptions {
    None,
    AToZ,
    ZToA,
    LowToHigh,
    HighToLow
}

impl StoreOptions {
    pub const LIST: [StoreOptions; 4] = [
        StoreOptions::All,
        StoreOptions::Steam,
        StoreOptions::GOG,
        StoreOptions::MicrosoftStore,
    ];
}

impl PriceOptions {
    pub const LIST: [PriceOptions; 5] = [
        PriceOptions::None,
        PriceOptions::Under5,
        PriceOptions::Under10,
        PriceOptions::Under25,
        PriceOptions::Custom,
    ];
}

impl SortOptions {
    pub const LIST: [SortOptions; 5] = [
        SortOptions::None,
        SortOptions::AToZ,
        SortOptions::ZToA,
        SortOptions::LowToHigh,
        SortOptions::HighToLow
    ];
}

impl std::fmt::Display for StoreOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreOptions::Steam => write!(f, "Steam"),
            StoreOptions::GOG => write!(f, "GOG"),
            StoreOptions::MicrosoftStore => write!(f, "Microsoft Store"),
            StoreOptions::All => write!(f, "All")
        }
    }
}

impl std::fmt::Display for PriceOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceOptions::None => write!(f,"None"),
            PriceOptions::Under5 => write!(f,"Under $5"),
            PriceOptions::Under10 => write!(f,"Under $10"),
            PriceOptions::Under25 => write!(f,"Under $25"),
            PriceOptions::Custom => write!(f,"Custom Range"),
        }
    }
}

impl std::fmt::Display for SortOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOptions::None => write!(f,"None"),
            SortOptions::AToZ => write!(f,"Title (A - Z)"),
            SortOptions::ZToA => write!(f,"Title (Z - A)"),
            SortOptions::LowToHigh => write!(f,"Price (Low - High)"),
            SortOptions::HighToLow => write!(f,"Price (High - Low)"),
        }
    }
}