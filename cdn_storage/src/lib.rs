#![feature(async_closure)]
#![feature(once_cell)]
#![deny(unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate tracing;

pub mod config;
mod delete;
mod download;
mod entrypoint;
pub mod node;
mod upload;

pub use delete::*;
pub use download::*;
pub use entrypoint::*;
pub use upload::*;
