use stretto::AsyncCache;

use std::lazy::SyncOnceCell as OnceCell;

use crate::config::{CACHE as CACHE_ENABLED, CACHE_SIZE};

pub static CACHE: OnceCell<AsyncCache<String, Vec<u8>>>  = OnceCell::new();

pub fn load_cache() {
    if !*CACHE_ENABLED {
        return;
    }

    let cache_size: i64 = *CACHE_SIZE;

    let max_counters: usize = (cache_size * 10) as usize;

    CACHE
        .set(
            AsyncCache::builder(max_counters, cache_size)
                .finalize()
                .unwrap_or_else(|e| panic!("Failed to initialize cache: {}", e)),
        )
        .unwrap_or_else(|_| panic!("Failed to initialize cache: did you call load_cache() twice?"));
}

pub async fn insert_into_cache(key: String, value: Vec<u8>, cost: i64) {
    CACHE
        .get()
        .unwrap_or_else(|| panic!("Cache not initialized: did you call load_cache()?"))
        .insert(key, value, cost)
        .await;
}

#[must_use]
pub fn get_from_cache(key: &str) -> Option<Vec<u8>> {
    let val = CACHE
        .get()
        .unwrap_or_else(|| panic!("Cache not initialized: did you call load_cache()?"))
        .get(&key.to_string());

    if val.is_some() {
        return Some(
            val.unwrap_or_else(|| unreachable!("Unwrap failed even if it passed is_some() guard."))
                .value()
                .clone(),
        );
    }

    None
}
