use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display};

pub struct Expr {
    pub op: char,
    pub b_sign: bool,
    pub pos: bool,
    pub vars: Box<BTreeSet<char>>,
}

impl Expr {
    pub fn new(op: char, b_sign: bool, pos: bool) -> Self {
        Expr {
            op,
            b_sign,
            pos,
            vars: Box::new(BTreeSet::new()),
        }
    }

    pub fn pref_suff(&self, p_str: &mut String, s_str: &mut String) {
        let closed_par = !self.pos || !self.b_sign;
        match (self.pos, self.b_sign) {
            (false, false) => *p_str = format!("-~({}", self.op) + p_str,
            (false, true) => *p_str = format!("-({}", self.op) + p_str,
            (true, false) => *p_str = format!("~({}", self.op) + p_str,
            _ => *p_str = format!("{}", self.op) + &p_str,
        };

        if closed_par {
            s_str.push(')');
        }
        s_str.push('/');
    }
}

pub enum ExprOperand {
    SubExpr(usize),
    Term(Term),
}

pub struct Term {
    pub val: ExprVal,
    pub b_sign: bool,
    pub pos: bool,
}

impl Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.pos, self.b_sign) {
            (false, false) => write!(f, "-~{:?}", self.val),
            (false, true) => write!(f, "-{:?}", self.val),
            (true, false) => write!(f, "~{:?}", self.val),
            _ => write!(f, "{:?}", self.val),
        }
    }
}

pub enum ExprVal {
    U32(u32),
    Var(char),
}

impl Debug for ExprVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprVal::U32(u) => write!(f, "{}", u),
            ExprVal::Var(c) => write!(f, "{}", c),
        }
    }
}
