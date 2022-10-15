use std::collections::VecDeque;

use std::mem;

use super::arena::Arena;
use super::node::VarTerm;
use super::utils::compute_sign;

use crate::error::ExprError;

pub struct ArenaFactory;
impl ArenaFactory {
    pub fn new_arena(rpn: &mut VecDeque<String>) -> Result<Arena, ExprError> {
        let mut arena = Arena::new();
        arena.root_node = ArenaFactory::build_expr(rpn, &mut arena)?;

        Ok(arena)
    }

    fn build_expr(rpn: &mut VecDeque<String>, arena: &mut Arena) -> Result<usize, ExprError> {
        fn take_node_stack(
            arena: &mut Arena,
            curr_node: &mut Option<usize>,
            node_stack: &mut VecDeque<usize>,
        ) {
            // take curr not on node_stack
            if let Some(idx) = curr_node.take() {
                node_stack.push_back(idx);
            }

            // clean node_stack -> node with enough children are merge/pushed onto parents
            while let Some(idx) = node_stack.pop_back() {
                let has_2_t = arena.get_num_terms(idx) >= 2;
                // ... it has at least 2children, it should be merge/push to top of node_stack
                if let (true, Some(&mut par_idx)) = (has_2_t, node_stack.back_mut()) {
                    let (par_op, par_sign) =
                        arena.copy_op_sign(par_idx).expect("should have found node");
                    let (e_op, e_sign) = arena.copy_op_sign(idx).expect("should have found node");
                    // if same op,sign as top of stack -> merge children
                    if par_op == e_op && par_sign == e_sign {
                        arena.move_ch(par_idx, idx).expect("ch not moved");
                    } else {
                        // ... if not set as child
                        arena.push_ch(par_idx, &vec![idx]).expect("ch not pushed");
                    }
                } else {
                    // ... else put back on top
                    node_stack.push_back(idx);
                    break;
                }
            }
        }

        println!("{}", format!("rpn: {:?}", rpn));

        if rpn.len() == 0 {
            return Err(ExprError::RPNEmpty());
        }

        let mut prev_leaf = false;
        let mut curr_node: Option<usize> = None;
        let mut sign_stack: Vec<String> = Vec::new();
        let mut sign = String::new();
        let mut node_stack: VecDeque<usize> = VecDeque::new();

        while let Some(elem) = rpn.pop_back() {
            let node_vec: Vec<String> = node_stack.iter().map(|i| (&arena).elem_str(*i)).collect();
            let node_str = curr_node.as_ref().and_then(|i| Some(arena.elem_str(*i)));
            println!(
                "{}",
                format!(
                    "{:?} - node_stack: {:?} - curr_node: {:?} - elem: {}",
                    rpn, node_vec, node_str, elem
                )
            );

            match elem.as_str() {
                "+" | "." | "^" | "&" | "|" => {
                    prev_leaf = false;
                    if let Some(new_op) = elem.chars().next() {
                        if let Some(idx) = curr_node {
                            let (curr_op, _) =
                                arena.copy_op_sign(idx).expect("should have found op.");
                            if curr_op == new_op && sign.len() == 0 {
                                continue;
                            }
                        }
                        if sign.len() > 0 {
                            sign_stack.push(mem::take(&mut sign));
                        }

                        take_node_stack(arena, &mut curr_node, &mut node_stack);

                        curr_node =
                            Some(arena.node(sign_stack.pop().unwrap_or("".to_string()), new_op));
                    } else {
                        unreachable!()
                    }
                }
                "~" | "-" | "-~" | "~-" => {
                    if sign.len() > 2 {
                        return Err(ExprError::RPNSyntaxError());
                    }
                    sign.push_str(&elem);
                }
                _ => {
                    if let Ok(c_u) = elem.parse::<u32>() {
                        let cst = compute_sign(c_u, mem::take(&mut sign));
                        match curr_node.as_mut() {
                            Some(idx) => arena
                                .push_cst(*idx, cst)
                                .expect("should have been able to push cst"),
                            None => return Err(ExprError::RPNSyntaxError()),
                        };
                    } else if let (true, Some(c_var)) = (elem.len() == 1, elem.chars().next()) {
                        let val = VarTerm {
                            val: c_var,
                            sign: mem::take(&mut sign),
                        };
                        let id_l = arena.leaf(val);
                        match curr_node.as_mut() {
                            Some(idx) => arena
                                .push_ch(*idx, &vec![id_l])
                                .expect("should have been able to push."),
                            None => {
                                if rpn.len() == 0 {
                                    return Ok(id_l);
                                }
                                return Err(ExprError::RPNSyntaxError());
                            }
                        }
                    } else {
                        return Err(ExprError::RPNSyntaxError());
                    }

                    if prev_leaf {
                        take_node_stack(arena, &mut curr_node, &mut node_stack);
                        curr_node = node_stack.pop_back();
                    }

                    sign = String::new();
                    prev_leaf = true;
                }
            }
        }

        if let Some(idx) = curr_node {
            return Ok(idx);
        }

        Ok(node_stack.pop_back().ok_or(ExprError::RPNSyntaxError())?)
    }
}
