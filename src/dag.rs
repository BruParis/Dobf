use crate::error::DAGError;
use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display};

pub trait DAGTrait: std::fmt::Debug {
    fn is_cst(&self) -> bool;
    fn bitwise(&self) -> bool;
}

pub enum DAGElem {
    Node(DAGNode),
    Leaf(DAGLeaf),
}

impl fmt::Debug for DAGElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DAGElem::Node(n) => write!(f, "{:?}", n),
            DAGElem::Leaf(l) => write!(f, "{:?}", l),
        };

        Ok(())
    }
}

impl DAGElem {
    fn is_cst(&self) -> bool {
        match self {
            DAGElem::Node(n) => n.is_cst(),
            DAGElem::Leaf(l) => l.is_cst(),
        }
    }
    pub fn is_mba(&self) -> bool {
        match self {
            DAGElem::Node(n) => n.is_mba(),
            DAGElem::Leaf(_) => true,
        }
    }
    fn is_mba_term(&self) -> bool {
        match self {
            DAGElem::Node(n) => n.is_mba_term(),
            DAGElem::Leaf(_) => true,
        }
    }
    pub fn valid(&self) -> bool {
        match self {
            DAGElem::Node(n) => n.valid(),
            DAGElem::Leaf(_) => true,
        }
    }

    pub fn bitwise(&self) -> bool {
        match self {
            DAGElem::Node(n) => n.bitwise(),
            DAGElem::Leaf(l) => l.bitwise(),
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
    pub value: DAGValue,
    pub b_sign: bool,
    pub pos: bool,
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

pub struct DAGNode {
    pub op: char,
    pub ch: Box<Vec<DAGElem>>,
    pub b_sign: bool,
    pub pos: bool,
    pub vars: Box<BTreeSet<char>>,
}

impl DAGNode {
    pub fn new(op: char, b_sign: bool, pos: bool) -> Result<Self, DAGError> {
        Ok(DAGNode {
            op,
            ch: Box::new(Vec::new()),
            b_sign,
            pos,
            vars: Box::new(BTreeSet::new()),
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

    pub fn valid(&self) -> bool {
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
