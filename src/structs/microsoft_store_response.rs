use std::collections::HashMap;
use clap::builder::Str;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ProductInfo {
    #[serde(rename = "productId")]
    pub product_id: String,
    pub title: String,
    description: String,
    categories: Vec<String>,
    #[serde(rename = "publisherName")]
    publisher_name: Option<String>,
    images: Vec<Image>,
    #[serde(rename = "averageRating")]
    average_rating: f64,
    price: f64,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "productFamilyName")]
    product_family_name: String,
    #[serde(rename = "packageFamilyNames")]
    package_family_names: Vec<String>,
    #[serde(rename = "isGamingAppOnly")]
    is_gaming_app_only: bool,
    installer: HashMap<String,String>,
    #[serde(rename = "skusSummary")]
    pub skus_summary: Vec<Option<SkusSummary>>,
    #[serde(rename = "releaseDateUtc")]
    release_date_utc: Option<String>,
    previews: Vec<Preview>,
    #[serde(rename = "priceInfo")]
    pub price_info: PriceInfo,
    #[serde(rename = "typeTag")]
    type_tag: String,
    #[serde(rename = "ratingCountFormatted")]
    rating_count_formatted: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "posterArtUrl")]
    poster_art_url: String,
    #[serde(rename = "boxArtUrl")]
    pub box_icon_url: String,
    #[serde(rename = "iconUrlBackground")]
    icon_url_background: String,
    screenshots: Vec<Screenshots>,
    #[serde(rename = "encodedTitle")]
    encoded_title: String,
    #[serde(rename = "isApplication")]
    is_application: bool,
    #[serde(rename = "isGame")]
    is_game: bool,
    #[serde(rename = "isTvSeries")]
    is_tv_series: bool,
    #[serde(rename = "isMovie")]
    is_movie: bool,
    #[serde(rename = "isMoviesOrTVs")]
    is_movies_or_tvs: bool,
    #[serde(rename = "isPwa")]
    is_pwa: bool,
    #[serde(rename = "isCoreGame")]
    is_core_game: bool,
    #[serde(rename = "isAllowed")]
    is_allowed: bool,
    #[serde(rename = "isBrowsable")]
    is_browsable: bool,
    #[serde(rename = "isPurchaseEnabled")]
    is_purchase_enabled: bool,
    #[serde(rename = "isAd")]
    is_ad: bool,
    #[serde(rename = "isSparkProduct")]
    is_spark_product: bool,
    #[serde(rename = "isAndroid")]
    is_android: bool,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: Option<String>,
    #[serde(rename = "isHardware")]
    is_hardware: bool,
    #[serde(rename = "isSubscription")]
    is_subscription: bool,
    #[serde(rename = "isTencent")]
    is_tencent: bool,
    #[serde(rename = "isTencentMini")]
    is_tencent_mini: bool,
    #[serde(rename = "pageTitleLocalization")]
    page_title_localization: String,
    #[serde(rename = "disableDownload")]
    disable_download: bool,
    #[serde(rename = "cardActions")]
    card_actions: Vec<String>,
    #[serde(rename = "productRatings")]
    product_ratings: Vec<ProductRating>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameInfo{
    #[serde(rename = "shortTitle")]
    short_title: String,
    #[serde(rename = "shortDescription")]
    short_description: String,
    #[serde(rename = "categoryId")]
    category_id: String,
    #[serde(rename = "categoryIds")]
    category_ids: Vec<String>,
    #[serde(rename = "approximateSizeInBytes")]
    approximate_size_in_bytes: i64,
    capabilities: Vec<String>,
    #[serde(rename = "developerName")]
    developer_name: String,
    #[serde(rename = "durationInSeconds")]
    duration_in_seconds: i64,
    #[serde(rename = "hasAddOns")]
    has_add_ons: bool,
    #[serde(rename = "hasThirdPartyIAPs")]
    has_third_party_iaps: bool,
    language: String,
    #[serde(rename = "maxInstallSizeInBytes")]
    max_install_size_in_bytes: i64,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(rename = "promoMessage")]
    promo_message: Option<String>,
    #[serde(rename = "publisherId")]
    publisher_id: String,
    #[serde(rename = "ratingCount")]
    rating_count: i32,
    #[serde(rename = "additionalLicenseTerms")]
    additional_license_terms: String,
    #[serde(rename = "appWebsiteUrl")]
    app_website_url: Option<String>,
    #[serde(rename = "disclaimerText")]
    disclaimer_text: String,
    features: Vec<String>,
    #[serde(rename = "gamingOptionsXboxLive")]
    gaming_options_xbox_live: bool,
    #[serde(rename = "installationTerms")]
    installation_terms: String,
    #[serde(rename = "isMicrosoftProduct")]
    is_microsoft_product: bool,
    #[serde(rename = "isMsixvc")]
    is_msixvc: bool,
    #[serde(rename = "lastUpdateDateUtc")]
    last_update_date_utc: String,
    #[serde(rename = "permissionsRequired")]
    permissions_required: Vec<String>,
    platforms: Option<Vec<String>>,
    #[serde(rename = "privacyUrl")]
    privacy_url: String,
    #[serde(rename = "productRatings")]
    product_ratings: Vec<ProductRating>,
    #[serde(rename = "publisherAddress")]
    publisher_address: String,
    #[serde(rename = "publisherCopyrightInformation")]
    publisher_copyright: String,
    #[serde(rename = "publisherPhoneNumber")]
    publisher_phone_number: String,
    #[serde(rename = "supportedLanguages")]
    supported_languages: Vec<String>,
    #[serde(rename = "supportUris")]
    support_uris: Vec<HashMap<String, String>>,
    #[serde(rename = "systemRequirements")]
    system_requirements: SystemRequirements,
    version: String,
    #[serde(rename = "warningMessage")]
    warning_message: Option<Vec<HashMap<String, String>>>,
    #[serde(rename = "pdpImageUrl")]
    pub pdp_image_url: String,
    #[serde(rename = "productId")]
    pub product_id: String,
    pub title: String,
    description: String,
    categories: Vec<String>,
    #[serde(rename = "publisherName")]
    publisher_name: Option<String>,
    images: Vec<Image>,
    #[serde(rename = "averageRating")]
    average_rating: f64,
    pub price: f64,
    #[serde(rename = "displayPrice")]
    pub display_price: String,
    #[serde(rename = "strikethroughPrice")]
    pub strikethrough_price: Option<String>,
    #[serde(rename = "productFamilyName")]
    product_family_name: String,
    #[serde(rename = "packageFamilyNames")]
    package_family_names: Vec<String>,
    #[serde(rename = "isGamingAppOnly")]
    is_gaming_app_only: bool,
    installer: HashMap<String,String>,
    #[serde(rename = "skusSummary")]
    pub skus_summary: Vec<Option<SkusSummary>>,
    #[serde(rename = "releaseDateUtc")]
    release_date_utc: Option<String>,
    #[serde(rename = "eventsInfo")]
    events_info: Vec<String>,
    previews: Vec<Preview>,
    #[serde(rename = "priceInfo")]
    pub price_info: PriceInfo,
    #[serde(rename = "ratingCountFormatted")]
    rating_count_formatted: Option<String>,
    #[serde(rename = "allowedPlatforms")]
    allowed_platforms: Vec<String>,
    #[serde(rename = "productType")]
    product_type: String,
    skus: Vec<SKU>,
    #[serde(rename = "catalogSource")]
    catalog_source: String,
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "posterArtUrl")]
    poster_art_url: String,
    #[serde(rename = "boxArtUrl")]
    pub box_icon_url: String,
    #[serde(rename = "heroImageUrl")]
    hero_image_url: Option<String>,
    #[serde(rename = "iconUrlBackground")]
    icon_url_background: String,
    trailers: Vec<Trailers>,
    screenshots: Vec<Screenshots>,
    #[serde(rename = "encodedTitle")]
    encoded_title: String,
    #[serde(rename = "isApplication")]
    is_application: bool,
    #[serde(rename = "isGame")]
    is_game: bool,
    #[serde(rename = "isTvSeries")]
    is_tv_series: bool,
    #[serde(rename = "isMovie")]
    is_movie: bool,
    #[serde(rename = "isMoviesOrTVs")]
    is_movies_or_tvs: bool,
    #[serde(rename = "isPwa")]
    is_pwa: bool,
    #[serde(rename = "isCoreGame")]
    is_core_game: bool,
    #[serde(rename = "isAllowed")]
    is_allowed: bool,
    #[serde(rename = "isBrowsable")]
    is_browsable: bool,
    #[serde(rename = "isPurchaseEnabled")]
    is_purchase_enabled: bool,
    #[serde(rename = "isAd")]
    is_ad: bool,
    #[serde(rename = "isSparkProduct")]
    is_spark_product: bool,
    #[serde(rename = "isAndroid")]
    is_android: bool,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: Option<String>,
    #[serde(rename = "isHardware")]
    is_hardware: bool,
    #[serde(rename = "isSubscription")]
    is_subscription: bool,
    #[serde(rename = "isTencent")]
    is_tencent: bool,
    #[serde(rename = "isTencentMini")]
    is_tencent_mini: bool,
    #[serde(rename = "pageTitleLocalization")]
    page_title_localization: String,
    #[serde(rename = "disableDownload")]
    disable_download: bool,
    #[serde(rename = "cardActions")]
    card_actions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Screenshots {
    #[serde(rename = "backgroundColor")]
    background_color: String,
    caption: String,
    #[serde(rename = "foregroundColor")]
    foreground_color: String,
    height: i32,
    #[serde(rename = "imagePositionInfo")]
    image_position_info: String,
    #[serde(rename = "imageType")]
    image_type: String,
    url: String,
    width: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Image {
    #[serde(rename = "imageType")]
    image_type: Option<String>,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
    #[serde(rename = "foregroundColor")]
    foreground_color: Option<String>,
    caption: Option<String>,
    #[serde(rename = "imagePositionInfo")]
    image_position_info: Option<String>,
    url: String,
    height: i32,
    width: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SkusSummary {
    msrp: Option<f64>,
    #[serde(rename = "displayRP")]
    display_msrp: Option<String>,
    #[serde(rename = "salePrices")]
    sale_prices: Option<Vec<SalePrice>>,
    included_with: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SalePrice {
    conditions: Option<Conditions>,
    price: f64,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "badgeId")]
    badge_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Conditions {
    #[serde(rename = "type")]
    condition_type: String,
    id: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Preview {
    #[serde(rename = "$type")]
    preview_type: String,
    #[serde(rename = "imageType")]
    image_type: Option<String>,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
    #[serde(rename = "foregroundColor")]
    foreground_color: Option<String>,
    caption: Option<String>,
    #[serde(rename = "imagePositionInfo")]
    image_position_info: Option<String>,
    url: String,
    height: i32,
    width: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceInfo {
    pub msrp: Option<f64>,
    pub price: Option<f64>,
    #[serde(rename = "badgeText")]
    pub badge_text: Option<String>,
    #[serde(rename = "forceToDisplayPrice")]
    force_to_display_price: bool,
    #[serde(rename = "narratorText")]
    narrator_text: String,
    ownership: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProductRating {
    #[serde(rename = "ratingSystem")]
    rating_system: String,
    #[serde(rename = "ratingSystemShortName")]
    rating_system_short_name: String,
    #[serde(rename = "ratingSystemId")]
    rating_system_id: String,
    #[serde(rename = "ratingSystemUrl")]
    rating_system_url: String,
    #[serde(rename = "ratingValue")]
    rating_value: String,
    #[serde(rename = "ratingId")]
    rating_id: String,
    #[serde(rename = "ratingValueLogoUrl")]
    rating_value_logo_url: String,
    #[serde(rename = "ratingAge")]
    rating_age: i64,
    #[serde(rename = "restrictMetadata")]
    restrict_metadata: bool,
    #[serde(rename = "restrictPurchase")]
    restrict_purchase: bool,
    #[serde(rename = "ratingDescriptors")]
    rating_descriptors: Option<Vec<String>>,
    #[serde(rename = "ratingDescriptorLogoUrls")]
    rating_descriptor_logo_urls: Option<Vec<String>>,
    #[serde(rename = "ratingDisclaimers")]
    rating_disclaimers: Option<Vec<HashMap<String, String>>>,
    #[serde(rename = "interactiveElements")]
    interactive_elements: Vec<String>,
    #[serde(rename = "longName")]
    long_name: String,
    #[serde(rename = "shortName")]
    short_name: String,
    description: String,
    #[serde(rename = "hasInAppPurchases")]
    has_in_app_purchases: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SystemRequirements {
    minimum: Minimum,
    recommended: Recommended,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Minimum {
    title: String,
    items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    level: String,
    #[serde(rename = "itemCode")]
    item_code: String,
    name: String,
    description: String,
    #[serde(rename = "isValidationPassed")]
    is_validation_passed: Option<bool>,
    priority: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Recommended {
    title: String,
    items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SKU{
    #[serde(rename = "skuId")]
    sku_id: String,
    actions: Vec<String>,
    #[serde(rename = "availabilityId")]
    availability_id: String,
    price: f64,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "fulfillmentData")]
    fulfillment_data: Option<String>,
    #[serde(rename = "skuType")]
    sku_type: String,
    msrp: Option<f64>,
    #[serde(rename = "displayMSRP")]
    display_msrp: Option<String>,
    #[serde(rename = "salePrices")]
    sale_prices: Option<Vec<SalePrice>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Trailers {
    title: String,
    #[serde(rename = "videoPurpose")]
    video_purpose: String,
    #[serde(rename = "audioEncoding")]
    audio_encoding: String,
    #[serde(rename = "videoEncoding")]
    video_encoding: String,
    image: Image,
    bitrate: i64,
    #[serde(rename = "videoPositionInfo")]
    video_position_info: String,
    #[serde(rename = "sortOrder")]
    sort_order: i32,
    url: String,
    height: i32,
    width: i32
}