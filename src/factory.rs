use std::collections::{BTreeSet, VecDeque};
use std::iter::FromIterator;

use crate::dag::{DAGElem, DAGLeaf, DAGNode, DAGValue};
use crate::error::DAGError;

pub struct DAGFactory;
impl DAGFactory {
    pub fn new_dag(rpn: &mut VecDeque<String>) -> Result<DAGElem, DAGError> {
        let mut dag = DAGFactory::build_dag(rpn)?;

        match dag {
            DAGElem::Node(ref mut n) => {
                DAGFactory::collect_vars(n);
                if !n.valid() {
                    return Err(DAGError::RPNSyntaxError());
                }
            }
            _ => (),
        }

        return Ok(dag);
    }

    fn collect_vars(node: &mut DAGNode) {
        println!("collect vars {:?}", node);
        node.vars = Box::new(
            node.ch
                .iter_mut()
                .map(|c| match c {
                    DAGElem::Node(n) => {
                        DAGFactory::collect_vars(n);
                        println!("  -> c.vars {:?}", n.vars);
                        Some(Vec::from_iter(n.vars.clone().into_iter()))
                    }
                    DAGElem::Leaf(l) => match l.value {
                        DAGValue::Var(v) => Some(vec![v]),
                        _ => None,
                    },
                })
                .flatten()
                .flatten()
                .collect::<BTreeSet<char>>(),
        );
    }

    fn push_ch(node: &mut DAGNode, ch: DAGElem) {
        node.ch.push(ch);
    }

    fn push_ch_vec(node: &mut DAGNode, ch: &mut Vec<DAGElem>) {
        node.ch.append(ch);
    }

    fn build_dag(rpn: &mut VecDeque<String>) -> Result<DAGElem, DAGError> {
        fn take_node_stack(
            curr_node: &mut Option<DAGNode>,
            node_stack: &mut VecDeque<DAGNode>,
            pop_stack: bool,
        ) {
            if let Some(mut node) = curr_node.take() {
                if let (true, Some(par_node)) = (node.ch.len() > 1, node_stack.back_mut()) {
                    if par_node.op == node.op {
                        DAGFactory::push_ch_vec(par_node, &mut node.ch);
                    } else {
                        DAGFactory::push_ch(par_node, DAGElem::Node(node));
                    }

                    if pop_stack {
                        *curr_node = node_stack.pop_back();
                    }
                } else {
                    node_stack.push_back(node);
                }
            }
        }
        println!("{}", format!("rpn: {:?}", rpn));

        if rpn.len() == 0 {
            return Err(DAGError::RPNEmpty());
        }

        let mut prev_leaf = false;
        let mut curr_node: Option<DAGNode> = None;
        let mut curr_b_sign = true;
        let mut curr_pos = true;
        let mut node_stack: VecDeque<DAGNode> = VecDeque::new();

        while let Some(elem) = rpn.pop_back() {
            println!(
                "{}",
                format!(
                    "{:?} - node_stack: {:?} - curr_node: {:?} - elem: {}",
                    rpn, node_stack, curr_node, elem
                )
            );
            match elem.as_str() {
                "+" | "-" | "." | "^" | "&" | "|" => {
                    prev_leaf = false;
                    if let Some(mut new_op) = elem.chars().next() {
                        let this_pos = curr_pos;
                        let neg = new_op == '-';
                        if neg {
                            new_op = '+';
                            curr_pos = false;
                        }

                        if let Some(node) = curr_node.as_ref() {
                            if node.op == new_op {
                                continue;
                            }
                        }

                        take_node_stack(&mut curr_node, &mut node_stack, false);

                        curr_node = Some(DAGNode::new(new_op, curr_b_sign, this_pos)?);
                    } else {
                        unreachable!()
                    }

                    curr_b_sign = true;
                }
                "~" => {
                    if !curr_b_sign {
                        return Err(DAGError::RPNSyntaxError());
                    }
                    curr_b_sign = false;
                }
                _ => {
                    let leaf: DAGLeaf;
                    if let Ok(term_u) = elem.parse::<u32>() {
                        leaf = DAGLeaf::new(DAGValue::U32(term_u), curr_b_sign, curr_pos);
                    } else if let (true, Some(c_var)) = (elem.len() == 1, elem.chars().next()) {
                        leaf = DAGLeaf::new(DAGValue::Var(c_var), curr_b_sign, curr_pos);
                    } else {
                        return Err(DAGError::RPNSyntaxError());
                    }

                    match curr_node.as_mut() {
                        Some(node) => DAGFactory::push_ch(node, DAGElem::Leaf(leaf)),
                        None => {
                            if rpn.len() == 0 {
                                return Ok(DAGElem::Leaf(leaf));
                            }
                            return Err(DAGError::RPNSyntaxError());
                        }
                    }

                    if prev_leaf {
                        take_node_stack(&mut curr_node, &mut node_stack, true);
                    }

                    curr_pos = true;
                    curr_b_sign = true;
                    prev_leaf = true;
                }
            }
        }

        println!(
            "-> {}",
            format!(
                "{:?} - node_stack: {:?} - curr_node: {:?}",
                rpn, node_stack, curr_node
            )
        );

        if let Some(node) = curr_node {
            return Ok(DAGElem::Node(node));
        }

        if let (true, Some(node)) = (node_stack.len() == 1, node_stack.pop_back()) {
            return Ok(DAGElem::Node(node));
        }

        Err(DAGError::RPNSyntaxError())
    }
}

mod test {
    use std::collections::BTreeSet;

    use super::{DAGElem, DAGFactory, DAGNode};
    use crate::error::DAGError;
    use crate::parser::parse_rpn;

    #[test]
    fn test_ok_vars() -> Result<(), DAGError> {
        let expr = "x^y^(t|y^(t+a))".to_string();
        let res = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
        let mut expected = BTreeSet::new();
        expected.insert('x');
        expected.insert('y');
        expected.insert('t');
        expected.insert('a');
        assert!(if let DAGElem::Node(n) = res {
            assert_eq!(*n.vars, expected);
            true
        } else {
            false
        });

        let expr = "(t+a)^123.a^(x+y)^(c+y)".to_string();
        let res = DAGFactory::new_dag(&mut parse_rpn(expr).unwrap())?;
        let mut expected = BTreeSet::new();
        expected.insert('a');
        expected.insert('c');
        expected.insert('t');
        expected.insert('x');
        expected.insert('y');
        assert!(if let DAGElem::Node(n) = res {
            assert_eq!(*n.vars, expected);
            true
        } else {
            false
        });

        Ok(())
    }
}
