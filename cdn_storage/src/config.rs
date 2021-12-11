use std::env;

lazy_static! {
    pub static ref HOST: String =
        env::var("FC_CDN_HOST").unwrap_or_else(|| "https://cdn.ferris.chat".to_string());
    pub static ref STORAGE_PATH: String =
        env::var("FC_CDN_UPLOADS_PATH").unwrap_or_else(|| "../../uploads".to_string());
}
