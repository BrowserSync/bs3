#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod fs;
#[cfg(not(target_arch = "wasm32"))]
pub mod resp;
#[cfg(not(target_arch = "wasm32"))]
pub mod start;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

#[cfg(target_arch = "wasm32")]
pub mod ws;
#[cfg(target_arch = "wasm32")]
pub use ws::client::*;

#[cfg(target_arch = "wasm32")]
pub fn main() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> std::io::Result<()> {
    start::main()
}
