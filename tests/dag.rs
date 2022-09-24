use dobf::error::DAGError;
use dobf::factory::DAGFactory;
use dobf::parser::parse_rpn;

#[test]
fn test_ok_simple() -> Result<(), DAGError> {
    let expr = "x".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "x".to_string();
    assert_eq!(format!("{:?}", dag), expected);
    let expr = "x+y".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "x+y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+z;y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "w+x+y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+z;y;x;w/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "a^x+y".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "+^x;a;y".to_string();
    let expected = "+y;^x;a//".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "a^(x+y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "^a;+y;x".to_string();
    let expected = "^+y;x/;a/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "(a+b+c)^(x+y+z)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "^a;+y;x".to_string();
    let expected = "^+z;y;x/;+c;b;a//".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "(a+b+c)^(x+y+(z^p))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "^a;+y;x".to_string();
    let expected = "^+^p;z/;y;x/;+c;b;a//".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "w+x+(a^b)+y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+z;y;^b;a/;x;w/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "a^(x+y.t)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "^a;+x;.t;y".to_string();
    let expected = "^+.t;y/;x/;a/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "123.a^(x+y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    //let expected = "^.a;123;+y;x".to_string();
    let expected = "^+y;x/;.a;123//".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    Ok(())
}

#[test]
fn test_ok_neg() -> Result<(), DAGError> {
    let expr = "x-y".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+-y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "x-y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+z;-y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "x-y+(z-t)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+-t;z;-y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "x-y+(z^(t-u))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+^+-u;t/;z/;-y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    let expr = "x-~y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "+z;-~y;x/".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    Ok(())
}

#[test]
fn test_ok_complex() -> Result<(), DAGError> {
    let expr = "(t+a)^123.a^(x+y)^(c+y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    let expected = "^+y;c/;+y;x/;.a;123/;+a;t//".to_string();
    assert_eq!(format!("{:?}", dag), expected);

    Ok(())
}

#[test]
fn test_bitwise() -> Result<(), DAGError> {
    let expr = "x+y+z+t".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), false);

    let expr = "x+y^a".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), false);

    let expr = "x^y^(t.y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), false);

    let expr = "x".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    Ok(())
}
