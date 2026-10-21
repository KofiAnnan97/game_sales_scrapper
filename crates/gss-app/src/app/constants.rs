pub const LOADING_FRAMES_SIZE: usize = 4;
pub const STATUS_ERR: &str = "ERROR";

// Search
pub const SKIP_STORE_SELECTION: usize = usize::MAX;
pub static MAX_RESULTS: usize = 20;

// Files
pub const NO_IMAGE: &[u8] = include_bytes!("../../resources/no_image.png");
pub static IMAGE_FROM_URL_TIMEOUT: u64 = 90;

// Logging
pub const BATCH_LIMIT: usize = 10;
pub const DEFAULT_LOGS_PER_PAGE: usize = 20;
pub const DEFAULT_MAX_CACHED_HISTORICAL_LOGS: usize = 5;
pub const DEFAULT_PAGES_PER_WINDOW: usize = 20;
pub const DEFAULT_ENTRIES_PER_WINDOW: usize = DEFAULT_LOGS_PER_PAGE * DEFAULT_PAGES_PER_WINDOW;
