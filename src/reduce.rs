use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use intbits::Bits;

use crate::expr::arena::Arena;
use crate::expr::utils::{compute_sign, fn_node};

fn reduce(arena: &mut Arena, root_idx: usize) {
    let mut node_idx_vec = vec![root_idx];

    while let Some(idx) = node_idx_vec.pop() {
        let mut ch_vec = arena.get_ch(idx);
        if !arena.is_mba(idx) {
            node_idx_vec.append(&mut ch_vec);
            continue;
        }

        let p_str = arena.elem_str(idx);
        print!("mba: {}", p_str);

        for ch_idx in ch_vec {
            if arena.is_bitwise(ch_idx) {
                fn_node(arena.get(idx), |n| {
                    if n.has_cst() {
                        panic!("bitwise expr should not have cst value!")
                    }
                });

                //let vars = arena.gather_vars(ch_idx);
                let pn = arena.pn(ch_idx);
                let tt = compute_truth_table(&pn);
            }
        }
    }
}

pub fn compute_truth_table(pn: &Vec<(char, String)>) -> Vec<u8> {
    let vars: BTreeSet<char> = pn
        .into_iter()
        .filter_map(|(c, _)| if "^|&!".contains(*c) { None } else { Some(*c) })
        .collect();

    let num_var = vars.len() as u8;
    if num_var > 3 {
        panic!(format!("num of vars: {} is too large for now", num_var))
    }

    let base: i32 = 2;
    let mut res = vec![];
    for i in 0..(base.pow(num_var as u32)) {
        let bool_map: BTreeMap<char, bool> = vars
            .clone()
            .into_iter()
            .enumerate()
            .map(|(pos, v)| (v, i.bit(pos)))
            .collect();

        let mut curr_pn = pn.clone();

        println!("START - {:?}", bool_map);

        let mut aux_op = '-';
        let mut aux_b: Option<bool> = Some(true);
        let mut stack = vec![];
        while let Some((c, sign)) = curr_pn.pop() {
            if sign.len() > 0 && sign != "~" {
                panic!("arithm. sign - in bitwise formula");
            }

            let bool_sign = sign == "~";

            match c {
                '^' | '|' | '&' | '!' => {
                    let mut val = aux_b.expect("should have found bool!");
                    match c {
                        '^' | '|' | '&' => {
                            if aux_op != '-' {
                                stack.push((val, aux_op, bool_sign));
                            }
                            aux_op = c;
                            aux_b = None;
                            println!("new op : {}", aux_op);
                        }
                        '!' => {
                            let (par_val, par_op, par_bool_s) = stack
                                .pop()
                                .expect("should have found stacked parent (value, op)");
                            val ^= par_bool_s;
                            aux_b = Some(compute_bool(par_op, par_val, val));
                            println!(
                                "{}",
                                format!(
                                    "op :{} - b1: {} - b2: {} - res: {:?}",
                                    par_op, par_val, val, aux_b
                                )
                            );
                            aux_op = par_op;
                        }
                        _ => unreachable!(),
                    }
                }
                var => {
                    let var_bool =
                        bool_sign ^ bool_map.get(&var).expect("should have found variable");

                    if let Some(b) = aux_b {
                        aux_b = Some(compute_bool(aux_op, b, var_bool));
                        println!(
                            "{}",
                            format!(
                                "op :{} - b1: {} - b2: {} - res: {:?}",
                                aux_op, b, var_bool, aux_b
                            )
                        );
                    } else {
                        aux_b = Some(var_bool);
                    }
                }
            };
        }

        let mut b = aux_b.expect("should have a bool value");
        while let Some((par_val, par_op, par_bool_s)) = stack.pop() {
            b ^= par_bool_s;
            println!("b: {}", b);
            b = compute_bool(par_op, par_val, b);
            println!(
                "{}",
                format!("op :{} - b1: {} --> res: {:?}", par_op, par_val, b)
            );
        }

        println!("aux_b {}", b);
        res.push(b as u8);
    }

    res
}

fn compute_bool(op: char, b1: bool, b2: bool) -> bool {
    match op {
        '^' => b1 ^ b2,
        '|' => b1 | b2,
        '&' => b1 & b2,
        _ => unreachable!(),
    }
}
