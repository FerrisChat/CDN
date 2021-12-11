pub use deadpool;
use deadpool::managed::{PoolConfig, Timeouts};
pub use deadpool_redis;
use deadpool_redis::{Config, Pool, Runtime};

pub use redis;
use redis::aio::PubSub;
use redis::{Client, RedisResult};
use std::lazy::SyncOnceCell as OnceCell;

use crate::config::FC_CDN_REDIS_URL;

pub static REDIS_MANAGER: OnceCell<Pool> = OnceCell::new();
pub static NODE_ID: OnceCell<u64> = OnceCell::new();

pub async fn load_redis(node_id: u64) {
    NODE_ID
        .set(node_id)
        .unwrap_or_else(|_| panic!("Failed to set node id: did you call load_redis twice"));

    let mut cfg = Config::from_url(FC_CDN_REDIS_URL.clone());

    cfg.pool = {
        use core::time::Duration;
        Some(PoolConfig {
            max_size: 1024,
            timeouts: Timeouts {
                wait: Some(Duration::from_secs(15)),
                create: Some(Duration::from_secs(10)),
                recycle: Some(Duration::from_secs(3)),
            },
        })
    };
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("failed to create pool");

    REDIS_MANAGER.set(pool).unwrap_or_else(|_| {
        panic!("failed to set Redis pool: did you call load_redis() twice?");
    });
}

pub async fn get_max_content_length() -> u64 {
    let pool = REDIS_MANAGER.get().unwrap_or_else(|| {
        panic!("Redis pool not initialized: did you call load_redis()?");
    });

    let mut conn = pool
        .get()
        .await
        .expect("failed to open database connection");

    redis::cmd("GET")
        .arg("max_content_length")
        .query_async::<_, u64>(&mut conn)
        .await
        .unwrap_or(1024 * 1024 * 10);
}

pub fn get_node_id() -> u64 {
    NODE_ID.get().unwrap_or_else(|| {
        panic!("Node id not initialized: did you call load_redis()?");
    })
}
