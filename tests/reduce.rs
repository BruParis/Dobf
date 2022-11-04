use dobf::reduce::{bit_pop, comb_bitmask, compute_truth_table, solve_sierpinski};
use std::collections::BTreeSet;

fn test_no_sign(v: Vec<char>, e: Vec<i8>) {
    let aux_s = String::new();
    let mut pn: Vec<(char, String)> = v.iter().map(|x| (*x, aux_s.clone())).collect();
    pn.reverse();
    let (res, _) = compute_truth_table(&pn);
    assert_eq!(res, e);
}

fn test_sign(v: Vec<char>, s: Vec<&str>, e: Vec<i8>) {
    let mut pn: Vec<(char, String)> = v
        .iter()
        .map(|c| c.clone())
        .zip(s.iter().map(|&aux_s| String::from(aux_s)))
        .collect();
    pn.reverse();
    let (res, _) = compute_truth_table(&pn);
    assert_eq!(res, e);
}

#[test]
fn test_truth_table_no_sign() {
    let expr_v = vec!['^', 'x', 'y', 'z'];
    let expected = vec![0, 1, 1, 0, 1, 0, 0, 1];
    test_no_sign(expr_v, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x'];
    let expected = vec![0, 1, 1, 1, 1, 0, 0, 0];
    test_no_sign(expr_v, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x', '!', 'x'];
    let expected = vec![0, 0, 1, 0, 1, 1, 0, 1];
    test_no_sign(expr_v, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x', '!', 'x', '&', 'x', 'y', 'z'];
    let expected = vec![0, 0, 1, 0, 1, 1, 0, 0];
    test_no_sign(expr_v, expected);
}

#[test]
fn test_truth_table_sign() {
    let expr_v = vec!['^', 'x', 'y', 'z'];
    let expr_s = vec!["", "~", "", ""];
    let expected = vec![1, 0, 0, 1, 0, 1, 1, 0];
    test_sign(expr_v, expr_s, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x'];
    let expr_s = vec!["", "", "", "", "~"];
    let expected = vec![1, 0, 1, 1, 0, 1, 0, 0];
    test_sign(expr_v, expr_s, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x', '!', 'x'];
    let expr_s = vec!["", "", "~", "", "", "", ""];
    let expected = vec![1, 1, 0, 1, 0, 0, 1, 0];
    test_sign(expr_v, expr_s, expected);

    let expr_v = vec!['^', 'z', '|', 'y', 'x', '!', 'x', '&', 'x', 'y', 'z'];
    let expr_s = vec!["", "", "", "", "", "", "", "~", "", "", ""];
    let expected = vec![1, 1, 0, 1, 0, 0, 1, 1];
    test_sign(expr_v, expr_s, expected);
}

fn test_comb_b(n_base: u8, bitmask: u64, exp_vec: Vec<u64>) {
    let expected = exp_vec.into_iter().collect();
    let set_comb: BTreeSet<u64> = comb_bitmask(n_base, bitmask).into_iter().collect();
    assert_eq!(set_comb, expected);
}

#[test]
fn test_comb_bitmask() {
    test_comb_b(3, 3, vec![0, 1, 2, 3]);
    test_comb_b(5, 20, vec![0, 4, 16, 20]);
    test_comb_b(5, 21, vec![0, 1, 4, 16, 5, 17, 20, 21]);
    test_comb_b(
        8,
        43,
        vec![0, 1, 2, 3, 8, 9, 32, 33, 10, 11, 35, 40, 34, 41, 42, 43],
    );
}

#[test]
fn test_bit_pop() {
    assert_eq!(bit_pop(3), 2);
    assert_eq!(bit_pop(21), 3);
    assert_eq!(bit_pop(43), 4);
    assert_eq!(bit_pop(63), 6);
}

fn test_solve_s(num_vars: u8, tt: Vec<i8>, exp: Vec<i8>) {
    let s = solve_sierpinski(tt, num_vars);
    assert_eq!(s, exp);
}

#[test]
fn test_solve_sierpinski() {
    let num_vars = 2;
    let tt = [0, 1, 0, 1];
    let expected = [0, 0, -1, 0, 1];
    test_solve_s(num_vars, tt.to_vec(), expected.to_vec());

    let tt = [1, 0, 1, 1];
    let expected = [-1, 0, 1, -1, 1];
    test_solve_s(num_vars, tt.to_vec(), expected.to_vec());

    let num_vars = 3;
    let tt = [1, 1, 1, 0, 1, 1, 1, 0];
    let expected = [0, 0, 0, 0, 1, 0, 0, -1, 1];
    test_solve_s(num_vars, tt.to_vec(), expected.to_vec());

    let tt = [1, 1, 0, 0, 0, 0, 1, 0];
    let expected = [1, -2, 0, 1, 0, 1, 0, -1, 1];
    test_solve_s(num_vars, tt.to_vec(), expected.to_vec());
}
