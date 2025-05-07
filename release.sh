rustup target add x86_64-unknown-linux-musl
rustup update

cargo clippy
cargo fmt
cargo build --target x86_64-unknown-linux-musl --release
cargo test --release
