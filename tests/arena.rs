use dobf::arena::{Arena, DAGFactory, Elem};
use dobf::error::ExprError;
use dobf::parser::parse_rpn;

fn get_expr_str(expr: String) -> Result<String, ExprError> {
    let mut arena = Arena::new();
    let elem_idx = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap(), &mut arena)?;
    Ok(format!("{}", arena.elem_str(elem_idx)))
}

#[test]
fn test_ok_simple() -> Result<(), ExprError> {
    let res = get_expr_str("x".to_string())?;
    assert_eq!(res, "x".to_string());

    let res = get_expr_str("x+y".to_string())?;
    assert_eq!(res, "+xy/".to_string());

    let res = get_expr_str("x+y+z".to_string())?;
    assert_eq!(res, "+xyz/".to_string());

    let res = get_expr_str("w+x+y+z".to_string())?;
    assert_eq!(res, "+wxyz/".to_string());

    let res = get_expr_str("a^x+y".to_string())?;
    assert_eq!(res, "+^ax/y/".to_string());

    let res = get_expr_str("a^(x+y)".to_string())?;
    assert_eq!(res, "^a+xy//".to_string());

    let res = get_expr_str("(a+b+c)^(x+y+z)".to_string())?;
    assert_eq!(res, "^+abc/+xyz//".to_string());

    let res = get_expr_str("(a+b+c)^(x+y)^z".to_string())?;
    assert_eq!(res, "^+abc/+xy/z/".to_string());

    let res = get_expr_str("(a+b+c)^(x+y+(z^p))".to_string())?;
    assert_eq!(res, "^+abc/+xy^zp///".to_string());

    let res = get_expr_str("w+x+(a^b)+y+z".to_string())?;
    assert_eq!(res, "+wx^ab/yz/".to_string());

    let res = get_expr_str("a^(x+y.t)".to_string())?;
    assert_eq!(res, "^a+x.yt///".to_string());

    let res = get_expr_str("123.a^(x+y)".to_string())?;
    assert_eq!(res, "^.123a/+xy//".to_string());

    Ok(())
}

#[test]
fn test_ok_neg() -> Result<(), ExprError> {
    let res = get_expr_str("x-y".to_string())?;
    assert_eq!(res, "+x-y/".to_string());

    let res = get_expr_str("x-y+z".to_string())?;
    assert_eq!(res, "+x-yz/".to_string());

    let res = get_expr_str("x-y+(z-t)".to_string())?;
    assert_eq!(res, "+x-yz-t/".to_string());

    let res = get_expr_str("x-y+(z^(t-u))".to_string())?;
    assert_eq!(res, "+^+-ut/z/-yx/".to_string());

    let res = get_expr_str("x-~y+z".to_string())?;
    assert_eq!(res, "+z-~yx/".to_string());

    Ok(())
}

#[test]
fn test_ok_complex() -> Result<(), ExprError> {
    let res = get_expr_str("(t+a)^123.a^(x+y)^(c+y)".to_string())?;
    assert_eq!(res, "^+ta/.123a/+xy/+cy//".to_string());

    Ok(())
}
