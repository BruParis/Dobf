use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display};

pub struct Expr {
    pub op: char,
    pub sign: String,
    pub vars: Box<BTreeSet<char>>,
}

impl Expr {
    pub fn new(op: char, sign: String) -> Self {
        Expr {
            op,
            sign,
            vars: Box::new(BTreeSet::new()),
        }
    }

    pub fn pref_suff(&self, p_str: &mut String, s_str: &mut String) {
        let closed_par = self.sign.len() > 0;
        if self.sign.len() > 0 {
            *p_str = format!("{}({}{}", &self.sign, &self.op, p_str);
        //*p_str = &self.sign + "(" + &self.op.to_string() + p_str;
        } else {
            *p_str = self.op.to_string() + p_str;
        }

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
    pub sign: String,
}

impl Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = format!("{}{:?}", self.sign, self.val);
        write!(f, "{}", res)
        /*match (self.pos, self.b_sign) {
            (false, false) => write!(f, "-~{:?}", self.val),
            (false, true) => write!(f, "-{:?}", self.val),
            (true, false) => write!(f, "~{:?}", self.val),
            _ => write!(f, "{:?}", self.val),
        }*/
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
