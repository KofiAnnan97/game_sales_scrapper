// Urls
pub static BASE_URL : &str = "https://apps.microsoft.com";

// Endpoints
pub static SEARCH_ENDPOINT : &str = "/api/products/search";
pub static PDP_ENDPOINT : &str = "/api/pages/pdp";

// Timeouts
pub static DEFAULT_TIMEOUT_IN_SECS : u64 = 20;

// Custom Error Messages
pub static MISSING_PRODUCTS_MSG : &str = "No products could be extracted from the API call.";