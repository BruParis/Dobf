use dobf::dag::DAGFactory;
use dobf::error::DAGError;
use dobf::parser::parse_rpn;

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

    let expr = "x^y^(t|y^(t|a))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y^(t+a))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), false);

    Ok(())
}

#[test]
fn test_mba() -> Result<(), DAGError> {
    let expr = "x+y.z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), false);

    let expr = "x+(z^(a+b))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), false);

    let expr = "x".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x^y".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x^(y|t)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+13".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y+z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y^z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y+4.z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y+4&z".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y+4.(z^x)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    let expr = "x+y+4.(z^(x|t))".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.is_mba(), true);

    Ok(())
}
