import solver_wasm, { solve, test } from "../../solver-wasm/pkg/solver_wasm.js"

const wasmReady = solver_wasm();

wasmReady.then(() => {
    // Hints strings needs to have lines separated by semi-colon and "blocks" separated by commas
    let hintsX = "2,2;4;1;2,1;1" 
    let hintsY = "1,2;2,1;1,1;2,1;2"    
    let dim = "5x5"
    
    const solution = solve(hintsX, hintsY, dim)

    test([1, 2, 3])
    
    // `solution` is the solved board as a string, e.g:
    //  
    // ██░██
    // ░████
    // █░░░░
    // ██░█░
    // ░░█░░
})