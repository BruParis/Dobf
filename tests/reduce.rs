use dobf::reduce::compute_truth_table;

fn test_no_sign(v: Vec<char>, e: Vec<u8>) {
    let aux_s = String::new();
    let mut pn: Vec<(char, String)> = v.iter().map(|x| (*x, aux_s.clone())).collect();
    pn.reverse();
    let res = compute_truth_table(&pn);
    assert_eq!(res, e);
}

#[test]
fn test_truth_table_no_sign() -> () {
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
