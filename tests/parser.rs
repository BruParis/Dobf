use dobf::error::ParseError;
use dobf::parser::parse_rpn;

#[test]
fn test_missing_par_simple() -> Result<(), ParseError> {
    let miss_cl = "(".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissClosePar("Missing )".to_string());
    assert_eq!(res, expected);

    let miss_cl = ")".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissOpenPar("Missing (".to_string());
    assert_eq!(res, expected);

    let miss_cl = "())".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissOpenPar("Missing (".to_string());
    assert_eq!(res, expected);

    let miss_cl = "(()".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissClosePar("Missing )".to_string());
    assert_eq!(res, expected);

    let miss_cl = "((x+y)".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissClosePar("Missing )".to_string());
    assert_eq!(res, expected);

    let miss_cl = "((x+(y^z)) + t".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::MissClosePar("Missing )".to_string());
    assert_eq!(res, expected);

    Ok(())
}

#[test]
fn test_succ_op() -> Result<(), ParseError> {
    let miss_cl = "+ ^".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::WrongSeqChar("wrong seq of char: +/^".to_string());
    assert_eq!(res, expected);

    let miss_cl = "(x+y) ^ z +((x - h)^z) -+ a".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::WrongSeqChar("wrong seq of char: -/+".to_string());
    assert_eq!(res, expected);

    Ok(())
}

#[test]
fn test_neg_sign_before_op() -> Result<(), ParseError> {
    let miss_cl = "~+".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::WrongSeqChar("wrong seq of char: ~/+".to_string());
    assert_eq!(res, expected);

    let miss_cl = "x + y - ~~|y".to_string();
    let res = parse_rpn(miss_cl).unwrap_err();
    let expected = ParseError::WrongSeqChar("wrong seq of char: ~/|".to_string());
    assert_eq!(res, expected);
    Ok(())
}

#[test]
fn test_ok_neg() -> Result<(), ParseError> {
    //need regex
    /*let miss_cl = "~~~y".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec!["y", "~"];
    assert_eq!(res, expected);*/

    /*let miss_cl = "x + y - ~~~(x^y)&((~x -y)^y)".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec![
        "x", "y", "+", "x", "y", "^", "~", "x", "~", "y", "-", "y", "^", "&", "-",
    ];
    assert_eq!(res, expected);*/
    Ok(())
}

#[test]
fn test_ok() -> Result<(), ParseError> {
    let miss_cl = "y+((((x+y)^z)))".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec!["y", "x", "y", "+", "z", "^", "+"];
    assert_eq!(res, expected);

    let miss_cl = "x+y-(x^y)".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec!["x", "y", "x", "y", "^", "-", "+", "+"];
    assert_eq!(res, expected);

    let miss_cl = "x + y -(x^y)&((x -y)^y)".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec![
        "x", "y", "x", "y", "^", "x", "y", "-", "+", "y", "^", "&", "-", "+", "+",
    ];
    assert_eq!(res, expected);

    let miss_cl = "x + (y) -(x^y+(~y))".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec!["x", "y", "x", "y", "^", "y", "~", "+", "-", "+", "+"];
    assert_eq!(res, expected);

    let miss_cl = "8458.(y&t&z) ^( x|y&z)&((x&y )& y|t) + x+ 9.(x|y)&y|z".to_string();
    let res = parse_rpn(miss_cl).unwrap();
    let expected = vec![
        "8458", "y", "t", "&", "z", "&", ".", "x", "y", "|", "z", "&", "x", "y", "&", "y", "&",
        "t", "|", "&", "^", "x", "+", "9", "x", "y", "|", ".", "y", "&", "z", "|", "+",
    ];
    assert_eq!(res, expected);

    Ok(())
}
