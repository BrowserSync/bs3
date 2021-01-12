#[cfg(not(target_arch = "wasm32"))]
pub mod browser_sync;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod fs;
#[cfg(not(target_arch = "wasm32"))]
pub mod resp;
#[cfg(not(target_arch = "wasm32"))]
pub mod start;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

#[cfg(not(target_arch = "wasm32"))]
pub mod serve_static;

#[cfg(not(target_arch = "wasm32"))]
pub mod routes;

#[cfg(not(target_arch = "wasm32"))]
mod local_url;
#[cfg(not(target_arch = "wasm32"))]
pub mod proxy;

#[cfg(not(target_arch = "wasm32"))]
mod bs_error;

#[cfg(target_arch = "wasm32")]
pub use ws::client::*;
#[cfg(target_arch = "wasm32")]
pub mod ws;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_utils;

#[cfg(not(target_arch = "wasm32"))]
mod server;

mod output;

#[cfg(not(target_arch = "wasm32"))]
pub mod json;

#[cfg(target_arch = "wasm32")]
pub fn main() {}
