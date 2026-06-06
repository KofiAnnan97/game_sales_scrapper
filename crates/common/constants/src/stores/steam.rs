// Filenames
pub static CACHE_FILENAME : &str = "cached_steam_games.json";

// Urls
pub static API_BASE_URL : &str = "https://api.steampowered.com";
pub static STORE_BASE_URL : &str = "https://store.steampowered.com";

// Endpoints
pub static APP_LIST_ENDPOINT : &str = "/IStoreService/GetAppList/v1";
pub static DETAILS_ENDPOINT : &str = "/api/appdetails";

//Store page
pub static STORE_PAGE_URL : &str = "https://store.steampowered.com/app/";

// Cache update parameters
pub static NUM_OF_RESULTS : u32 = 40000;
pub static SLIDING_UPDATE_START_SIZE : usize = 100000;

// Search settings
pub static SIMPLE_SEARCH : &str = "simple";
pub static FUZZY_SEARCH : &str = "fuzzy";
pub static DEFAULT_SEARCH_TYPE : &str = FUZZY_SEARCH;
pub static SEARCH_SIZE_LIMIT : usize = 100;