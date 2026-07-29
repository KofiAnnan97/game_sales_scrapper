// Api Version
pub static VERSION: u32 = 2;

// Urls
pub static BASE_URL_V1 : &str = "https://embed.gog.com";
pub static BASE_URL_V2 : &str = "https://catalog.gog.com";

// Endpoints
pub static MEDIA_ENDPOINT_V1 : &str = "/games/ajax/filtered";
pub static CATALOG_ENDPOINT_V2 : &str = "/v1/catalog";

// Timeouts
pub static DEFAULT_TIMEOUT_IN_SECS : u64 = 20;

// Search limits
pub static SEARCH_LIMIT : u32 = 50;
pub static SINGLE_SEARCH : u32 = 1;


// Custom Error Messages
pub static MISSING_PRODUCTS_MSG : &str = "No products could be extracted from the API call.";