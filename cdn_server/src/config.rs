use std::env;
use sysinfo::System;

fn get_default_cache_size() -> i64 {
    let mut sys = System::new();
    sys.refresh_all();
    let sys_mem = sys.total_memory() * 1024;
    (sys_mem * 0.25) as i64
}

lazy_static! {
    pub static ref REDIS_URL: String =
        env::var("FC_CDN_REDIS_URL").expect("FC_CDN_REDIS_URL env var is not set.");
    pub static ref CACHE: bool = env::var("FC_CDN_CACHE")
        .map(|v| v == "true")
        .unwrap_or(false);
    pub static ref CACHE_SIZE: i64 = env::var("FC_CDN_CACHE_SIZE")
        .map(|v| v
            .parse::<i64>()
            .unwrap_or_else(|_| get_default_cache_size()))
        .unwrap_or_else(|_| get_default_cache_size());
}
