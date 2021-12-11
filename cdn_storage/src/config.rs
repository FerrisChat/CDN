use std::env;

lazy_static! {
    pub static ref HOST: String =
        env::var("FC_CDN_HOST").unwrap_or_else(|_| "https://cdn.ferris.chat".to_string());
    pub static ref STORAGE_PATH: String =
        env::var("FC_CDN_UPLOADS_PATH").unwrap_or_else(|_| "../../uploads".to_string());
    pub static ref REDIS_URL: String =
        env::var("FC_CDN_REDIS_URL").except("FC_CDN_REDIS_URL env var is not set.");
}
