# Asteroids

Trying out the `bevy` game engine.

## Compile and run

```sh
cargo run
```

*Warning: The first compilation will be quite slow, because the engine is compiled with full optimizations. Any later compilations/runs (for the same target) will take < 5s.*

## Build for Web

You need `wasm-bindgen-cli` and the `wasm32-unknown-unknown` target installed. (`cargo install wasm-bindgen-cli` and `rustup target add wasm32-unknown-unknown`)

```sh
cargo build --profile release-wasm --target wasm32-unknown-unknown
wasm-bindgen --out-name asteroids --out-dir target --target web target/wasm32-unknown-unknown/release-wasm/asteroids.wasm
```

Then open `asteroids.html` in a browser (you might need an http server, e.g. `python3 -m http.server --directory .`)

## Build for Android

You need the Android SDK, `cargo-apk`, the appropriate Rust targets (`aarch64-linux-android` and `armv7-linux-androideabi`), and maybe some other stuff installed.

1. Rename `src/main.rs` to `src/lib.rs`
2. Uncomment to section labeled `UNCOMMENT FOR ANDROID` in `Cargo.toml`
3. Use `cargo apk build` to build the APK
4. Install the app from `target/debug/apk/asteroids.apk`
