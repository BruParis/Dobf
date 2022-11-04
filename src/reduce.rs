use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use intbits::Bits;
use itertools::Itertools;

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
                let (tt, n_vars) = compute_truth_table(&pn);

                solve_sierpinski(tt, n_vars);
            }
        }
    }
}

// TODO: genericity on int type depending on number of vars
pub fn bit_pop(n: u64) -> u8 {
    (0..u64::N_BITS).map(|b| n.bit(b) as u8).sum()
}

pub fn solve_sierpinski(tt: Vec<i8>, num_vars: u8) -> Vec<i8> {
    let mut a = vec![tt[0]];
    for exp in 0..(num_vars) {
        let p2: u64 = 2 << exp;
        let mid = (p2 / 2) as u64;

        for i in 0..mid {
            let mut a_i = -a[i as usize];
            println!("mid: {}", mid);
            println!("{:?}", comb_bitmask(exp, i));
            for b in comb_bitmask(exp, i) {
                let b_xor_i = b ^ i;
                let s_pos = bit_pop(b_xor_i) % 2 == 0;
                let sign = (if s_pos { 1 } else { -1 }) as i8;
                a_i += tt[(mid + b) as usize] * sign;
            }

            a.push(a_i);
        }
    }
    a.reverse();
    let mut res: Vec<i8> = a.iter().map(|v| -v).collect();
    res.push(1);

    res
}

// Given a bit representation (of len n)...
// ... output all combination of its bit population
pub fn comb_bitmask(num_bits: u8, n: u64) -> Vec<u64> {
    println!("{}", format!("num_bits: {:?}-{}", num_bits, n));
    let bit_base: Vec<u8> = (0..num_bits)
        .filter_map(|b| if n.bit(b) { Some(b) } else { None })
        .collect();

    let bit_pop = bit_base.len();
    (0..(bit_pop + 1))
        .map(|n_b| {
            bit_base
                .iter()
                .combinations(n_b)
                .map(|comb| {
                    let mut i: u64 = 0;
                    for b in comb {
                        i.set_bit(*b, true);
                    }
                    i
                })
                .collect::<Vec<u64>>()
        })
        .flatten()
        .collect()
}

pub fn compute_truth_table(pn: &Vec<(char, String)>) -> (Vec<i8>, u8) {
    if pn.len() == 0 {
        panic!("empty polish notation");
    }

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
                        }
                        '!' => {
                            let (par_val, par_op, par_bool_s) = stack
                                .pop()
                                .expect("should have found stacked parent (value, op)");
                            val ^= par_bool_s;
                            aux_b = Some(compute_bool(par_op, par_val, val));
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
                    } else {
                        aux_b = Some(var_bool);
                    }
                }
            };
        }

        let mut b = aux_b.expect("should have a bool value");
        while let Some((par_val, par_op, par_bool_s)) = stack.pop() {
            b = compute_bool(par_op, par_val, b) ^ par_bool_s;
        }

        println!("aux_b {}", b);
        res.push(b as i8);
    }

    (res, num_var)
}

fn compute_bool(op: char, b1: bool, b2: bool) -> bool {
    match op {
        '^' => b1 ^ b2,
        '|' => b1 | b2,
        '&' => b1 & b2,
        _ => unreachable!(),
    }
}
