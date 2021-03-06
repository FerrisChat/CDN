pub use deadpool;
use deadpool::managed::{PoolConfig, Timeouts};
pub use deadpool_redis;
use deadpool_redis::{Config, Pool, Runtime};

pub use redis;
use std::lazy::SyncOnceCell as OnceCell;

use cdn_common::CdnError;

use crate::config::REDIS_URL;

pub static REDIS_MANAGER: OnceCell<Pool> = OnceCell::new();

pub fn load_redis() {
    let mut cfg = Config::from_url(REDIS_URL.clone());

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

pub async fn get_node_ip(node_id: String) -> Result<String, CdnError> {
    let pool = REDIS_MANAGER.get().unwrap_or_else(|| {
        panic!("Redis pool not initialized: did you call load_redis()?");
    });

    let mut conn = pool
        .get()
        .await
        .map_err(CdnError::FailedToOpenRedisConnection)?;

    Ok(redis::cmd("HGET")
        .arg("cdn_nodes")
        .arg(node_id)
        .query_async::<_, String>(&mut conn)
        .await
        .map_err(|_| CdnError::FailedToGetNode)?)
}

pub async fn get_all_nodes() -> Result<Vec<String>, CdnError> {
    let pool = REDIS_MANAGER.get().unwrap_or_else(|| {
        panic!("Redis pool not initialized: did you call load_redis()?");
    });

    let mut conn = pool
        .get()
        .await
        .map_err(CdnError::FailedToOpenRedisConnection)?;

    Ok(redis::cmd("HKEYS")
        .arg("cdn_nodes")
        .query_async::<_, Vec<String>>(&mut conn)
        .await
        .map_err(|_| CdnError::FailedToGetNode)?)
}
