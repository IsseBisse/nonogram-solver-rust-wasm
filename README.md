Simple nonogram solver. Built in Rust and meant to compile to WASM.

# Build and usage
Run `wasm-pack build --target web` from inside the `solver-wasm/` directory. Add the files in the generated `pkg/` directory to your project and call the solver as shown in the `example.js` file.

## Performance benchmark
Run `cargo bench` from inside `solver-wasm/` to run a performance benchmark. Add `SAVE_BENCH=1` to save results to `solver-wasm/benches/results/`. 

**NOTE: Results will be marked with commit hash, so make sure all changes have been committed!**