set -euxo pipefail

cargo fix --allow-dirty --allow-staged && cargo fmt
cargo clippy
cargo check

cd bs3_core
sed -i.bak 's/#crate-type/crate-type/' Cargo.toml
wasm-pack build --dev
sed -i.bak 's/crate-type/#crate-type/' Cargo.toml
