# Staking-contract
Staking contract on ink rust


Run the contract through.

rustup toolchain install nightly-2022-08-15
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-15
rustup component add rust-src --toolchain nightly-2022-08-15
cargo +nightly-2022-08-15 contract build


cargo-contract latest release currently is v1.5.0 which was released on 15th August 2022. So I used rust nightly build of that day and it worked.
