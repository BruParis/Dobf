use std::fmt::{self, Debug};

use super::expr::{Expr, Other};
use super::utils::compute_op;

pub struct VarTerm {
    pub val: char,
    pub sign: String,
}

impl Debug for VarTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.sign, self.val)
    }
}

pub struct Node {
    pub(super) idx: usize,
    sign: String,
    pub(super) expr: Expr,
    pub(super) cst: Option<u32>,
    pub(super) par: Option<usize>,
}

impl Node {
    pub(super) fn new(idx: usize, sign: String, op: char) -> Self {
        Node {
            idx,
            sign,
            expr: Expr::Other(Other::new(op)),
            cst: None,
            par: None,
        }
    }

    pub(super) fn push_cst(&mut self, cst: u32) {
        if let Some(old_cst) = self.cst.take() {
            self.cst = Some(compute_op(self.expr.op(), old_cst, cst));
        } else {
            self.cst = Some(cst);
        }
    }

    pub(super) fn graph_label_str(&self) -> String {
        let main_str = format!("{} [label=<{}{}>]\n", self.idx, self.sign, self.expr.op());

        if let Some(cst) = self.cst {
            let cst_node = format!("\"{}c\" [label=<{}>]\n", self.idx, cst);
            format!("{}{}", main_str, cst_node)
        } else {
            main_str
        }
    }

    pub(super) fn graph_edge_str(&self) -> String {
        let main_str = self
            .expr
            .ch()
            .iter()
            .map(|c| format!("{} -> {}\n", self.idx, c))
            .collect::<Vec<String>>()
            .join("");
        if let Some(_) = self.cst {
            let cst_arrow = format!("{} -> \"{}c\"\n", self.idx, self.idx);
            format!("{}{}", main_str, cst_arrow)
        } else {
            main_str
        }
    }

    pub(super) fn op(&self) -> char {
        self.expr.op()
    }

    pub(super) fn sign(&self) -> String {
        self.sign.clone()
    }

    pub(super) fn ch(&self) -> Vec<usize> {
        self.expr.ch()
    }

    pub fn has_cst(&self) -> bool {
        !self.cst.is_none()
    }

    pub(super) fn pref_suff(&self) -> (String, String) {
        let (mut p_str, mut s_str) = (String::new(), String::new());
        let closed_par = self.sign.len() > 0;
        if self.sign.len() > 0 {
            p_str = format!("{}({}{}", &self.sign, &self.op(), p_str);
        } else {
            p_str = format!("{}{}", self.op().to_string(), p_str);
        }

        if let Some(cst) = self.cst {
            p_str.push_str(&cst.to_string());
        }

        if closed_par {
            s_str.push(')');
        }
        s_str.push('/');

        (p_str, s_str)
    }
}

pub struct Leaf {
    pub idx: usize,
    pub val: VarTerm,
    pub(super) par: Option<usize>,
}

impl Leaf {
    pub(super) fn graph_label_str(&self) -> String {
        format!(
            "{} [label=<{}{:?}>]\n",
            self.idx, self.val.sign, self.val.val
        )
    }
}
