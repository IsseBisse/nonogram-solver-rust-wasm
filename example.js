import init, { solve } from "../../solver-wasm/pkg/solver_wasm.js"

const wasmReady = init();

wasmReady.then(() => {
    let hintsX = "2,2;4;1;2,1;1"
    let hintsY = "1,2;2,1;1,1;2,1;2"    
    let dim = "5x5"
    
    const solution = solve(hintsX, hintsY, dim)
    
    // `solution` is the solved board as a string, e.g:
    //  
    // ██░██
    // ░████
    // █░░░░
    // ██░█░
    // ░░█░░
    
})