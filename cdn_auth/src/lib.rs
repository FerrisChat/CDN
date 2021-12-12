#![feature(once_cell)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate tracing;

mod auth;
mod config;
mod init;
mod split_token;
mod verify_token;

pub use argon2_async::{hash, verify, Error as Argon2Error};
pub use auth::*;
pub use config::*;
pub use init::init_auth;
pub use split_token::*;
pub use verify_token::*;

pub use cdn_common::{SplitTokenError, VerifyTokenFailure};

pub use sqlx;

use config::POSTGRES_URL;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::lazy::SyncOnceCell as OnceCell;
use std::time::Duration;

pub static DATABASE_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

/// Load the Postgres pool, set it into the global database pool, and return it.
///
/// # Panics
/// If the global pool was already set.
/// This will only happen if this function is called more than once.
pub async fn load_db() -> Pool<Postgres> {
    let db = PgPoolOptions::new()
        .max_connections(512)
        .min_connections(2)
        .max_lifetime(Some(Duration::from_secs(30 * 60)))
        .connect(POSTGRES_URL.clone().as_ref())
        .await
        // don't ask
        .unwrap_or_else(|_| panic!("failed to connect to DB"));

    DATABASE_POOL
        .set(db.clone())
        // also don't ask
        .unwrap_or_else(|_| panic!("failed to set the DB global: did you call load_db() twice?"));

    db
}
