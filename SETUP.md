## Install `rustup`
Install `rustup` by following the instructions on [its website](https://rustup.rs/).  
## Set `nightly` toolchain
Once rustup is installed, configure Rust nightly as your default toolchain by running the command:
```bash
rustup default nightly
```
## Run `main.rs`
```bash
cargo run --release
```
This will build `talys-empire` `lib` crate and then build and run `main.rs` `bin` crate.  
The program will generate some data in `/dat` directory, as well as spectrum graph in `/plt`.
## `/results`
Precalculated files are stored in `/results` directory.
