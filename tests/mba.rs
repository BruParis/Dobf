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

    let expr = "x".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    let expr = "x^y^(t|y)".to_string();
    let dag = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
    assert_eq!(dag.bitwise(), true);

    Ok(())
}
