use solver_wasm::solve;

mod tests {
    use super::*;

    #[test]
    fn test_solver() {
        let hints_x_str = "1,2,3;4,5,6";
        let hints_y_str = "7,8;9,10,11";

        solve(hints_x_str, hints_y_str);
    }
}