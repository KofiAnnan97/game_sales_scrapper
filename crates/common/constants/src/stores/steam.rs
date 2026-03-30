// Filenames
pub static CACHE_FILENAME : &str = "cached_steam_games.json";

// Urls
pub static API_BASE_URL : &str = "https://api.steampowered.com";
pub static STORE_BASE_URL : &str = "https://store.steampowered.com";

// Endpoints
pub static APP_LIST_ENDPOINT : &str = "/IStoreService/GetAppList/v1";
pub static DETAILS_ENDPOINT : &str = "/api/appdetails";

// Cache update parameters
pub static NUM_OF_RESULTS : u32 = 40000;
pub static SLIDING_UPDATE_START_SIZE : usize = 100000;