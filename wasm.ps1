$ErrorActionPreference = 'Stop'
cargo fmt
cargo build --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --out-dir ./web --target web --no-typescript ./target/wasm32-unknown-unknown/wasm-release/misty_maze.wasm
wasm-opt -Oz ./web/misty_maze_bg.wasm -o ./web/misty_maze_bg.wasm
