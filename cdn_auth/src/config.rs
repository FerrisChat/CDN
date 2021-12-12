use std::env;

lazy_static! {
    pub static ref POSTGRES_URL: String =
        env::var("FC_CDN_POSTGRES_URL").expect("FC_CDN_POSTGRES_URL env var is not set.");
}
