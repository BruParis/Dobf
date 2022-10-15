use std::collections::VecDeque;
use std::mem;

use super::expr::Expr;
use super::node::{Leaf, Node, VarTerm};
use super::utils::{compute_op, compute_sign, match_elem, match_elem_mut, node_is_bitwise};

use crate::error::ArenaError;

pub struct Arena {
    pub root_node: usize,
    elems: Vec<Elem>,
    free_slots: Vec<usize>,
}

pub enum Elem {
    Node(Node),
    Leaf(Leaf),
    Free,
}

impl Default for Elem {
    fn default() -> Self {
        Elem::Free
    }
}

impl Elem {
    fn get_par_id(&self) -> Option<usize> {
        match_elem(self, |n| n.par, |l| l.par)
    }

    fn graph_label_str(&self) -> String {
        match_elem(self, |n| n.graph_label_str(), |l| l.graph_label_str())
    }
    fn graph_edge_str(&self) -> Option<String> {
        match_elem(self, |n| Some(n.graph_edge_str()), |_| None)
    }
}

impl Arena {
    pub fn new() -> Self {
        Arena {
            root_node: 0,
            elems: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub fn node(&mut self, sign: String, op: char) -> usize {
        // TODO: check if node already registered
        let idx = self.len();
        let node = Node::new(idx, sign, op);
        self.elems.push(Elem::Node(node));
        idx
    }

    pub fn leaf(&mut self, val: VarTerm) -> usize {
        // TODO: check if leaf already registered
        let idx = self.len();
        let leaf = Leaf {
            idx,
            val,
            par: None,
        };
        self.elems.push(Elem::Leaf(leaf));
        idx
    }

    fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn get(&self, idx: usize) -> &Elem {
        self.elems.get(idx).expect("elem not found at index!")
    }

    pub fn get_node(&self, idx: usize) -> Result<&Node, ArenaError> {
        let e = self.get(idx);

        match e {
            Elem::Node(n) => Ok(n),
            _ => Err(ArenaError::NotANode()),
        }
    }

    pub fn get_ch(&self, idx: usize) -> Vec<usize> {
        match_elem(self.get(idx), |n| n.ch(), |_| vec![])
    }

    pub fn get_ch_len(&self, idx: usize) -> usize {
        match_elem(self.get(idx), |n| n.ch().len(), |_| 0)
    }

    pub fn get_num_terms(&self, idx: usize) -> usize {
        match_elem(
            self.get(idx),
            |n| {
                if n.cst.is_none() {
                    n.ch().len()
                } else {
                    n.ch().len() + 1
                }
            },
            |_| 0,
        )
    }

    pub(super) fn copy_op_sign(&self, idx: usize) -> Option<(char, String)> {
        match_elem(self.get(idx), |n| Some((n.op(), n.sign())), |_| None)
    }

    fn take_cst(&mut self, idx: usize) -> Option<u32> {
        match_elem_mut(self.get_mut(idx), &mut |n| n.cst.take(), &mut |_| None)
    }

    fn take_ch(&mut self, idx: usize) -> Option<Vec<usize>> {
        match_elem_mut(
            self.get_mut(idx),
            &mut |n| Some(n.expr.take_ch()),
            &mut |_| None,
        )
    }

    fn get_mut(&mut self, idx: usize) -> &mut Elem {
        self.elems.get_mut(idx).expect("elem not found at index!")
    }

    fn remove_elem(&mut self, idx: usize) -> Result<Elem, ArenaError> {
        let rem_elem = mem::replace(self.get_mut(idx), Elem::Free);
        self.free_slots.push(idx);
        Ok(rem_elem)
    }

    pub fn get_preorder(&self, idx: usize, only_node: bool) -> Vec<usize> {
        let mut idx_stack = vec![idx];
        let mut res: Vec<usize> = Vec::new();
        while let Some(idx) = idx_stack.pop() {
            if only_node & self.is_leaf(idx) {
                continue;
            }

            res.push(idx);
            let e = self.get(idx);
            match_elem(e, |n| idx_stack.append(&mut n.ch().clone()), |_| ())
        }

        res
    }

    pub fn graph_str(&self) -> String {
        let graph_label_str = self
            .elems
            .iter()
            .map(Elem::graph_label_str)
            .collect::<Vec<String>>()
            .join("");
        let graph_edge_str = self
            .elems
            .iter()
            .flat_map(Elem::graph_edge_str)
            .collect::<Vec<String>>()
            .join("");
        "digraph {\n".to_string() + &graph_label_str + &graph_edge_str + "}"
    }

    pub fn print(&self) -> String {
        self.elem_str(self.root_node)
    }

    pub fn elem_str(&self, idx: usize) -> String {
        let mut str_stack: VecDeque<String> = VecDeque::new();
        let mut suff_stack: Vec<String> = Vec::new();
        let mut par_idx: Vec<usize> = Vec::new();
        let mut res = String::new();

        let mut pre_order = self.get_preorder(idx, false);
        pre_order.reverse();

        while let Some(idx) = pre_order.pop() {
            let e = self.get(idx);

            // Not elegant
            if let Some(p_id) = e.get_par_id() {
                while let Some(&aux_id) = par_idx.last() {
                    // if current elem's parent id is different current parent,
                    // then parsing of all current parent children is done
                    // -> push parent suffix, change current parent
                    if p_id != aux_id {
                        res.push_str(&suff_stack.pop().expect("should have found suffix"));
                        par_idx.pop();
                    } else {
                        break;
                    }
                }
            }

            match_elem(e, |_| par_idx.push(idx), |_| ());
            let (p_str, s_str) = match_elem(
                e,
                |n| n.pref_suff(),
                |l| (format!("{:?}", l.val), "".to_string()),
            );
            res.push_str(&p_str);
            if s_str.len() > 0 {
                suff_stack.push(s_str);
            }
        }

        res.push_str(&suff_stack.join(""));

        while let Some(r_str) = str_stack.pop_front() {
            res.push_str(&r_str);
        }

        res
    }

    fn set_par(&mut self, idx: usize, par_id: usize) {
        match_elem_mut(
            self.get_mut(idx),
            &mut |n: &mut Node| n.par = Some(par_id),
            &mut |l: &mut Leaf| l.par = Some(par_id),
        )
    }

    // pre-cond: children are supposed to have had their cst taken if
    // compatible with expr
    fn push_ch_expr(&self, expr: &mut Expr, ch_idx: &Vec<usize>) {
        if let Some(new_expr) = expr.push_ch_mut(self, ch_idx) {
            mem::replace(expr, new_expr);
        }
    }

    fn push_ch_node(&mut self, node: &mut Node, ch_idx: &Vec<usize>) {
        // take all compatible cst from future ch
        // regarding sign and op associativity
        // and compute them when current node's cst
        //node.cst = self.filter_cst(&mut node.expr, node.cst, &ch_idx);
        node.cst = {
            let op = node.expr.op();
            ch_idx
                .iter()
                .filter_map(|&idx| {
                    if let Some((ch_op, ch_sign)) = self.copy_op_sign(idx) {
                        if op == ch_op {
                            if let Some(ch_cst) = self.take_cst(idx) {
                                return Some(compute_sign(ch_cst, ch_sign));
                            }
                        }
                    }
                    None
                })
                .fold(node.cst, |acc, cst| {
                    if let Some(acc_v) = acc {
                        Some(compute_op(node.expr.op(), acc_v, cst))
                    } else {
                        Some(cst)
                    }
                })
        };

        // all adequate ch's have been taken
        // push all ch to new parent
        self.push_ch_expr(&mut node.expr, &ch_idx);

        // set new parent to all new children
        for idx in ch_idx {
            self.set_par(*idx, node.idx);
        }
    }

    pub(super) fn push_ch(
        &mut self,
        idx: usize,
        idx_ch_vec: &Vec<usize>,
    ) -> Result<(), ArenaError> {
        let mut temp_n = mem::take(self.get_mut(idx));
        match_elem_mut(
            &mut temp_n,
            &mut |n| {
                self.push_ch_node(n, &idx_ch_vec);
                Ok(())
            },
            &mut |_| Err(ArenaError::ParentIsLeaf()),
        )?;

        self.elems[idx] = temp_n;

        Ok(())
    }

    pub(super) fn push_cst(&mut self, idx: usize, cst: u32) -> Result<(), ArenaError> {
        match_elem_mut(
            &mut self.get_mut(idx),
            &mut |n| {
                n.push_cst(cst);
                Ok(())
            },
            &mut |_| Err(ArenaError::ParentIsLeaf()),
        )?;

        Ok(())
    }
    pub(super) fn move_ch(&mut self, idx_to: usize, idx_from: usize) -> Result<(), ArenaError> {
        let from_ch: Vec<usize>;

        if let Some(ch) = self.take_ch(idx_from) {
            from_ch = ch;
        } else {
            return Err(ArenaError::NotANode());
        }

        self.push_ch(idx_to, &from_ch)?;

        // update children's new parent id
        for ch_idx in from_ch {
            match_elem_mut(
                self.get_mut(ch_idx),
                &mut |n| n.par = Some(idx_to),
                &mut |l| l.par = Some(idx_to),
            );
        }

        self.remove_elem(idx_from)?;

        Ok(())
    }

    fn is_leaf(&self, idx: usize) -> bool {
        match_elem(self.get(idx), |_| false, |_| true)
    }

    pub fn is_bitwise(&self, idx: usize) -> bool {
        match_elem(self.get(idx), |n| node_is_bitwise(&n.expr, n.cst), |_| true)
    }

    pub fn is_mba(&self, idx: usize) -> bool {
        match_elem(self.get(idx), |n| matches!(n.expr, Expr::MBA(_)), |_| false)
    }
}
