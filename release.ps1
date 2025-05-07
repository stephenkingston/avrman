rustup target add x86_64-pc-windows-msvc
rustup update

cargo clippy
cargo fmt
cargo build --target x86_64-pc-windows-msvc --release
