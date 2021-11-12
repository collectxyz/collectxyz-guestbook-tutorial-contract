# xyz Guestbook Tutorial Contract

This repository contains an example smart contract that illustrates how to build on top of the [xyz NFT contract](https://github.com/collectxyz/collectxyz-nft-contract).

This contract implements a "guestbook" that allows the owner of an xyz to store a single guestbook entry at that xyz's current coordinate location for a small fee. If the xyz owner relocates their xyz, they can make another guestbook entry at their xyz's new location.

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Testing

Run all tests for the workspace:

```sh
cargo test
```

### Compiling

To compile the NFT contract, run:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
shasum -a 256  target/wasm32-unknown-unknown/release/collectxyz_guestbook_tutorial_contract.wasm
```

#### Production

For production builds, first install run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.11.5

# or, install cargo-run-script, then run
cargo run-script optimize
```

This uses [rust-optimizer](https://github.com/cosmwasm/rust-optimizer) to perform several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.
