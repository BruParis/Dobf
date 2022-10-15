use dobf::error::ExprError;
use dobf::expr::factory::ArenaFactory;
use dobf::parser::parse_rpn;

fn get_expr_str(expr: String) -> Result<String, ExprError> {
    let arena = ArenaFactory::new_arena(&mut parse_rpn(expr).unwrap())?;
    Ok(format!("{}", arena.print()))
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
    assert_eq!(res, "+x-y^z+t-u///".to_string());

    let res = get_expr_str("x-~y+z".to_string())?;
    assert_eq!(res, "+x-~yz/".to_string());

    Ok(())
}

#[test]
fn test_ok_complex() -> Result<(), ExprError> {
    let res = get_expr_str("(t+a)^123.a^(x+y)^(c+y)".to_string())?;
    assert_eq!(res, "^+ta/.123a/+xy/+cy//".to_string());

    Ok(())
}

#[test]
fn test_from_ok_parser() -> Result<(), ExprError> {
    let res = get_expr_str("y+((((x+y)^z)))".to_string())?;
    assert_eq!(res, "+y^+xy/z//".to_string());

    let res = get_expr_str("x+y-(x^y)".to_string())?;
    assert_eq!(res, "+xy-(^xy/)/".to_string());

    let res = get_expr_str("x + y -(x^y)&((x -y)^y)".to_string())?;
    assert_eq!(res, "+xy-(&^xy/^+x-y/y/)//".to_string());

    let res = get_expr_str("x + (y) -(x^y+(~y))".to_string())?;
    assert_eq!(res, "+xy-(+^xy/~y/)/".to_string());

    let res = get_expr_str("8458.(y&t&z) ^( x|y&z)&((x&y )& y|t) + x+ 9.(x|y)&y|z".to_string())?;
    assert_eq!(
        res,
        "+.8458&ytz//^|xy/&z|&xyy/t///x|&.9|xy//y/z//".to_string()
    );

    Ok(())
}
