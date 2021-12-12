use std::env;

lazy_static! {
    pub static ref REDIS_URL: String =
        env::var("FC_CDN_REDIS_URL").expect("FC_CDN_REDIS_URL env var is not set.");
}
