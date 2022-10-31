use super::arena::{Arena, Elem};
use super::expr::Expr;
use super::node::{Leaf, Node};

pub(super) fn compute_op(op: char, cst1: u32, cst2: u32) -> u32 {
    match op {
        '+' => cst1 + cst2,
        '.' => cst1 * cst2,
        '^' => cst1 ^ cst2,
        '|' => cst1 | cst2,
        '&' => cst1 & cst2,
        _ => unreachable!(),
    }
}

pub fn compute_sign(cst: u32, sign: String) -> u32 {
    match sign.as_str() {
        "" => cst,
        "-" => (-(cst as i32)) as u32,
        "~" => !cst,
        "~-" => !((-(cst as i32)) as u32),
        "-~" => (-((!cst) as i32)) as u32,
        _ => unreachable!(),
    }
}

pub(super) fn is_mba(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if op != '+' {
        return false;
    }
    ch.iter().all(|&ch| {
        match_elem(
            arena.get(ch),
            |n| node_is_bitwise(&n.expr, n.cst) || node_is_mba_term(&n.expr, n.cst),
            |_| true,
        )
    })
}

pub(super) fn node_is_mba_term(expr: &Expr, cst: Option<u32>) -> bool {
    matches!(expr, Expr::MBATerm(_)) && !cst.is_none()
}

pub(super) fn node_is_bitwise(expr: &Expr, cst: Option<u32>) -> bool {
    matches!(expr, Expr::Bitwise(_)) && cst.is_none()
}

fn ch_bitwise(arena: &Arena, ch: &Vec<usize>) -> bool {
    ch.iter()
        .all(|&ch| match_elem(arena.get(ch), |n| node_is_bitwise(&n.expr, n.cst), |_| true))
}

pub(super) fn is_bitwise(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if !"^&|".contains(op) {
        return false;
    }
    ch_bitwise(arena, ch)
}

pub(super) fn is_mba_term(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if op != '.' {
        return false;
    }

    if ch.len() != 1 {
        return false;
    }

    ch_bitwise(arena, ch)
}

fn fn_elem<F, R>(elem: &Elem, func: F) -> R
where
    F: Fn(&Elem) -> R,
{
    match elem {
        Elem::Free => panic!("idx -> free elem"),
        e => func(e),
    }
}

pub fn fn_node<F, R>(elem: &Elem, func: F) -> R
where
    F: FnOnce(&Node) -> R,
{
    match elem {
        Elem::Node(n) => func(n),
        Elem::Leaf(_) => panic!("idx -> leaf elem"),
        Elem::Free => panic!("idx -> free elem"),
    }
}

pub fn fn_node_mut<F, R>(elem: &mut Elem, func: &mut F) -> R
where
    F: FnMut(&mut Node) -> R,
{
    match elem {
        Elem::Node(n) => func(n),
        Elem::Leaf(_) => panic!("idx -> leaf elem"),
        Elem::Free => panic!("idx -> free elem"),
    }
}

pub fn match_elem<FNo, Fl, R>(elem: &Elem, func_n: FNo, func_l: Fl) -> R
where
    FNo: FnOnce(&Node) -> R,
    Fl: FnOnce(&Leaf) -> R,
{
    match elem {
        Elem::Node(n) => func_n(n),
        Elem::Leaf(l) => func_l(l),
        _ => panic!("idx -> free elem"),
    }
}

pub(super) fn match_elem_mut<FNo, Fl, R>(elem: &mut Elem, func_n: &mut FNo, func_l: &mut Fl) -> R
where
    FNo: FnMut(&mut Node) -> R,
    Fl: FnMut(&mut Leaf) -> R,
{
    match elem {
        Elem::Node(n) => func_n(n),
        Elem::Leaf(l) => func_l(l),
        _ => panic!("idx -> free elem"),
    }
}
