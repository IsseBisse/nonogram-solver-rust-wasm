Simple nonogram solver. Built in Rust and meant to compile to WASM.

# Build and usage
Run `wasm-pack build --target web` from inside the `solver-wasm/` directory. Add the files in the generated `pkg/` directory to your project and call the solved as shown in the `example.js` file.