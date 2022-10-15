use dobf::error::ArenaError;
use dobf::expr::factory::ArenaFactory;
use dobf::parser::parse_rpn;

fn is_bitwise(expr: String) -> bool {
    let arena =
        ArenaFactory::new_arena(&mut parse_rpn(expr).unwrap()).expect("rpn should be valid");
    arena.is_bitwise(arena.root_node)
}

fn is_mba(expr: String) -> bool {
    let arena =
        ArenaFactory::new_arena(&mut parse_rpn(expr).unwrap()).expect("rpn should be valid");
    arena.is_mba(arena.root_node)
}

#[test]
fn test_bitwise() -> Result<(), ArenaError> {
    let expr = "x+y+z+t".to_string();
    assert_eq!(is_bitwise(expr), false);

    let expr = "32^a".to_string();
    assert_eq!(is_bitwise(expr), false);

    let expr = "x+y^a".to_string();
    assert_eq!(is_bitwise(expr), false);

    let expr = "x^y^(t.y)".to_string();
    assert_eq!(is_bitwise(expr), false);

    let expr = "x^y^(t|y^(t|a))".to_string();
    assert_eq!(is_bitwise(expr), true);

    let expr = "x".to_string();
    assert_eq!(is_bitwise(expr), true);

    let expr = "x^y^(t|y)".to_string();
    assert_eq!(is_bitwise(expr), true);

    let expr = "x^y^(t|y)".to_string();
    assert_eq!(is_bitwise(expr), true);

    let expr = "x^y^(t|y^(t+a))".to_string();
    assert_eq!(is_bitwise(expr), false);

    Ok(())
}

#[test]
fn test_mba() -> Result<(), ArenaError> {
    let expr = "x+y.z".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+(z^(a+b))".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+(z^4)".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x^y".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "3.(x^y)".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+4.(z^3.x)".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+y+4&z".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+y+4.z.a".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+y+4.(z^x).(a^b)".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x".to_string();
    assert_eq!(is_mba(expr), false);

    let expr = "x+y".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+x^y".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "a+x^(y|t)".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+13".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+y+z".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+y^z".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+y+4.z".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+y+4.(z^x)".to_string();
    assert_eq!(is_mba(expr), true);

    let expr = "x+y+4.(z^(x|t))".to_string();
    assert_eq!(is_mba(expr), true);

    Ok(())
}
