use crate::error::DAGError;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::iter::FromIterator;

pub trait DAGTrait: std::fmt::Debug {
    fn is_cst(&self) -> bool;
    fn bitwise(&self) -> bool;
}

pub enum DAGEnum {
    Node(Box<DAGNode>),
    Leaf(DAGLeaf),
}

impl fmt::Debug for DAGEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DAGEnum::Node(n) => write!(f, "{:?}", n),
            DAGEnum::Leaf(l) => write!(f, "{:?}", l),
        };

        Ok(())
    }
}

impl DAGEnum {
    fn is_cst(&self) -> bool {
        match self {
            DAGEnum::Node(n) => n.is_cst(),
            DAGEnum::Leaf(l) => l.is_cst(),
        }
    }
    pub fn is_mba(&self) -> bool {
        match self {
            DAGEnum::Node(n) => n.is_mba(),
            DAGEnum::Leaf(l) => true,
        }
    }
    fn is_mba_term(&self) -> bool {
        match self {
            DAGEnum::Node(n) => n.is_mba_term(),
            DAGEnum::Leaf(_) => true,
        }
    }
    fn valid(&self) -> bool {
        match self {
            DAGEnum::Node(n) => n.valid(),
            DAGEnum::Leaf(l) => true,
        }
    }

    pub fn bitwise(&self) -> bool {
        match self {
            DAGEnum::Node(n) => n.bitwise(),
            DAGEnum::Leaf(l) => l.bitwise(),
        }
    }
}

#[derive(Debug)]
pub enum DAGValue {
    U32(u32),
    Var(char),
}

impl Display for DAGValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DAGValue::U32(u) => write!(f, "{}", u),
            DAGValue::Var(c) => write!(f, "{}", c),
        }
    }
}

pub struct DAGLeaf {
    value: DAGValue,
    b_sign: bool,
    pos: bool,
}

impl DAGLeaf {
    pub fn new(value: DAGValue, b_sign: bool, pos: bool) -> Self {
        DAGLeaf { value, b_sign, pos }
    }
}

impl Debug for DAGLeaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.pos, self.b_sign) {
            (false, false) => write!(f, "-~{}", self.value),
            (false, true) => write!(f, "-{}", self.value),
            (true, false) => write!(f, "~{}", self.value),
            _ => write!(f, "{}", self.value),
        }
    }
}

impl DAGTrait for DAGLeaf {
    fn is_cst(&self) -> bool {
        match self.value {
            DAGValue::U32(_) => true,
            DAGValue::Var(_) => false,
        }
    }
    fn bitwise(&self) -> bool {
        match self.value {
            DAGValue::U32(_) => false,
            DAGValue::Var(_) => true,
        }
    }
}

#[derive(Debug)]
struct ExpVar {
    var: char,
}

pub struct DAGNode {
    op: char,
    ch: Box<Vec<DAGEnum>>,
    b_sign: bool,
    pos: bool,
    vars: BTreeSet<char>,
}

impl DAGNode {
    pub fn new(op: char, b_sign: bool, pos: bool) -> Result<Self, DAGError> {
        Ok(DAGNode {
            op,
            ch: Box::new(Vec::new()),
            b_sign,
            pos,
            vars: BTreeSet::new(),
        })
    }
    fn is_mba(&self) -> bool {
        match self.op {
            '+' => self.ch.iter().all(|ch| ch.is_mba_term()),
            '.' => self.is_mba_term(),
            _ => self.ch.iter().all(|ch| ch.bitwise()),
        }
    }

    fn is_mba_term(&self) -> bool {
        match self.op {
            '+' => false,
            '.' => {
                let mut node_count = 0;
                return self.ch.iter().all(move |ch| {
                    if ch.is_cst() {
                        return true;
                    }
                    node_count += 1;
                    if node_count > 1 {
                        return false;
                    }

                    return ch.bitwise();
                });
            }
            _ => self.ch.iter().all(|ch| ch.bitwise()),
        }
    }

    fn valid(&self) -> bool {
        if self.ch.len() < 2 {
            return false;
        }

        self.ch.iter().all(|ch| ch.valid())
    }
}

impl DAGTrait for DAGNode {
    fn is_cst(&self) -> bool {
        self.ch.iter().all(|ch| ch.is_cst())
    }

    fn bitwise(&self) -> bool {
        if "+.".contains(self.op) {
            return false;
        }
        self.ch.iter().all(|ch| ch.bitwise())
    }
}

impl fmt::Debug for DAGNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closed_par = !self.pos || !self.b_sign;
        match (self.pos, self.b_sign) {
            (false, false) => write!(f, "-~({}", self.op),
            (false, true) => write!(f, "-({}", self.op),
            (true, false) => write!(f, "~({}", self.op),
            _ => write!(f, "{}", self.op),
        };

        let mut it_ch = self.ch.iter().peekable();
        while let Some(ch) = it_ch.next() {
            if it_ch.peek().is_none() {
                write!(f, "{:#?}", ch);
            } else {
                write!(f, "{:#?};", ch);
            }
        }

        if closed_par {
            write!(f, ")/");
        }

        write!(f, "/");

        Ok(())
    }
}

pub struct DAGFactory;
impl DAGFactory {
    pub fn new_dag(rpn: &mut VecDeque<String>) -> Result<DAGEnum, DAGError> {
        let mut dag = DAGFactory::build_dag(rpn)?;

        match dag {
            DAGEnum::Node(ref mut n) => {
                DAGFactory::collect_vars(n.as_mut());
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
        node.vars = node
            .ch
            .iter_mut()
            .map(|c| match c {
                DAGEnum::Node(n) => {
                    DAGFactory::collect_vars(n.as_mut());
                    println!("  -> c.vars {:?}", n.vars);
                    Some(Vec::from_iter(n.vars.clone()))
                }
                DAGEnum::Leaf(l) => match l.value {
                    DAGValue::Var(v) => Some(vec![v]),
                    _ => None,
                },
            })
            .flatten()
            .flatten()
            .collect::<BTreeSet<char>>();
    }

    fn push_ch(node: &mut DAGNode, ch: DAGEnum) {
        node.ch.push(ch);
    }

    fn push_ch_vec(node: &mut DAGNode, ch: &mut Vec<DAGEnum>) {
        node.ch.append(ch);
    }

    fn build_dag(rpn: &mut VecDeque<String>) -> Result<DAGEnum, DAGError> {
        fn take_node_stack(
            curr_node: &mut Option<Box<DAGNode>>,
            node_stack: &mut VecDeque<Box<DAGNode>>,
            pop_stack: bool,
        ) {
            if let Some(mut node) = curr_node.take() {
                if let (true, Some(par_node)) = (node.ch.len() > 1, node_stack.back_mut()) {
                    if par_node.op == node.op {
                        DAGFactory::push_ch_vec(par_node, &mut node.ch);
                    } else {
                        DAGFactory::push_ch(par_node, DAGEnum::Node(node));
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
        let mut curr_node: Option<Box<DAGNode>> = None;
        let mut curr_b_sign = true;
        let mut curr_pos = true;
        let mut node_stack: VecDeque<Box<DAGNode>> = VecDeque::new();

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

                        if let Some(node) = curr_node.as_deref() {
                            if node.op == new_op {
                                continue;
                            }
                        }

                        take_node_stack(&mut curr_node, &mut node_stack, false);

                        curr_node = Some(Box::new(DAGNode::new(new_op, curr_b_sign, this_pos)?));
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
                        Some(node) => DAGFactory::push_ch(node, DAGEnum::Leaf(leaf)),
                        None => {
                            if rpn.len() == 0 {
                                return Ok(DAGEnum::Leaf(leaf));
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
            return Ok(DAGEnum::Node(node));
        }

        if let (true, Some(node)) = (node_stack.len() == 1, node_stack.pop_back()) {
            return Ok(DAGEnum::Node(node));
        }

        Err(DAGError::RPNSyntaxError())
    }
}

mod test {
    use std::collections::BTreeSet;

    use super::{DAGEnum, DAGFactory, DAGNode};
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
        assert!(if let DAGEnum::Node(n) = res {
            assert_eq!(n.vars, expected);
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
        assert!(if let DAGEnum::Node(n) = res {
            assert_eq!(n.vars, expected);
            true
        } else {
            false
        });

        Ok(())
    }
}
