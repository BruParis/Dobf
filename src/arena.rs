use std::collections::{BTreeSet, VecDeque};
use std::mem;

use crate::error::{ArenaError, ExprError};
use crate::expr::{Expr, ExprVal, Term};

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

fn fn_elem<F, R>(elem: &Elem, func: F) -> R
where
    F: Fn(&Elem) -> R,
{
    match elem {
        Elem::Free => panic!("idx -> free elem"),
        e => func(e),
    }
}

pub fn fn_node<F, R>(elem: &Elem, func: F) -> R
where
    F: FnOnce(&Node) -> R,
{
    match elem {
        Elem::Node(n) => func(n),
        Elem::Leaf(_) => panic!("idx -> leaf elem"),
        Elem::Free => panic!("idx -> free elem"),
    }
}

pub fn match_elem<FNo, Fl, R>(elem: &Elem, func_n: FNo, func_l: Fl) -> R
where
    FNo: FnOnce(&Node) -> R,
    Fl: FnOnce(&Leaf) -> R,
{
    match elem {
        Elem::Node(n) => func_n(n),
        Elem::Leaf(l) => func_l(l),
        _ => panic!("idx -> free elem"),
    }
}

fn match_elem_mut<FNo, Fl, R>(elem: &mut Elem, func_n: &mut FNo, func_l: &mut Fl) -> R
where
    FNo: FnMut(&mut Node) -> R,
    Fl: FnMut(&mut Leaf) -> R,
{
    match elem {
        Elem::Node(n) => func_n(n),
        Elem::Leaf(l) => func_l(l),
        _ => panic!("idx -> free elem"),
    }
}

impl Elem {
    fn get_par_id(&self) -> Option<usize> {
        match_elem(self, |n| n.par, |l| l.par)
    }

    fn get_idx_sign(&self) -> (usize, String) {
        match_elem(
            self,
            |n| (n.idx, n.val.sign.clone()),
            |l| (l.idx, l.val.sign.clone()),
        )
    }

    fn graph_label_str(&self) -> String {
        match_elem(self, |n| n.graph_label_str(), |l| l.graph_label_str())
    }
    fn graph_edge_str(&self) -> Option<String> {
        match_elem(self, |n| Some(n.graph_edge_str()), |_| None)
    }
}

pub struct Node {
    pub idx: usize,
    pub val: Expr,
    pub ch: Vec<usize>,
    par: Option<usize>,
}

impl Node {
    fn new(idx: usize, val: Expr) -> Self {
        Node {
            idx,
            val,
            ch: vec![],
            par: None,
        }
    }

    fn graph_label_str(&self) -> String {
        format!("{} [label=<{}{}>]\n", self.idx, self.val.sign, self.val.op)
    }

    fn graph_edge_str(&self) -> String {
        self.ch
            .iter()
            .map(|c| format!("{} -> {}\n", self.idx, c))
            .collect::<Vec<String>>()
            .join("")
    }
}

pub struct Leaf {
    pub idx: usize,
    pub val: Term,
    par: Option<usize>,
}

impl Leaf {
    fn graph_label_str(&self) -> String {
        format!(
            "{} [label=<{}{:?}>]\n",
            self.idx, self.val.sign, self.val.val
        )
    }

    fn is_bitwise(&self) -> bool {
        match self.val.val {
            ExprVal::U32(_) => false,
            ExprVal::Var(_) => true,
        }
    }

    fn is_cst(&self) -> bool {
        match self.val.val {
            ExprVal::U32(_) => true,
            ExprVal::Var(_) => false,
        }
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

    pub fn node(&mut self, val: Expr) -> usize {
        // TODO: check if node already registered
        let idx = self.len();
        let node = Node::new(idx, val);
        self.elems.push(Elem::Node(node));
        idx
    }

    pub fn leaf(&mut self, val: Term) -> usize {
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
        let e = self.get(idx);

        match e {
            Elem::Node(n) => n.ch.clone(),
            _ => vec![],
        }
    }

    fn get_op_sign(&self, idx: usize) -> Result<(char, String), ArenaError> {
        match self.get(idx) {
            Elem::Node(n) => Ok((n.val.op, n.val.sign.clone())),
            _ => Err(ArenaError::ElemIsLeaf()),
        }
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
            match_elem(e, |n| idx_stack.append(&mut n.ch.clone()), |_| ())
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
                |n| n.val.pref_suff(),
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

    fn push_ch(&mut self, idx: usize, idx_ch: usize) -> Result<(), ArenaError> {
        let mut par_idx_opt = None;
        match_elem_mut(
            self.get_mut(idx),
            &mut |n| {
                par_idx_opt = Some(n.idx);
                n.ch.push(idx_ch);
                Ok(())
            },
            &mut |_| Err(ArenaError::ParentIsLeaf()),
        )?;
        self.set_par(idx_ch, par_idx_opt.expect("should have found parent id"));
        Ok(())
    }

    fn push_ch_vec(&mut self, idx: usize, idx_ch_vec: &mut Vec<usize>) -> Result<(), ArenaError> {
        let mut par_idx_opt = None;
        match_elem_mut(
            self.get_mut(idx),
            &mut |n| {
                par_idx_opt = Some(n.idx);
                n.ch.append(idx_ch_vec);
                Ok(())
            },
            &mut |_| Err(ArenaError::ParentIsLeaf()),
        )?;

        for idx_ch in idx_ch_vec {
            self.set_par(*idx_ch, par_idx_opt.expect("should have found parent id"));
        }
        Ok(())
    }

    fn move_ch(&mut self, idx_to: usize, idx_from: usize) -> Result<(), ArenaError> {
        let from_ch: Vec<usize>;
        match self.get(idx_from) {
            Elem::Node(from) => from_ch = from.ch.clone(),
            _ => return Err(ArenaError::NotANode()),
        }

        match self.get_mut(idx_to) {
            Elem::Node(to) => {
                to.ch.append(&mut from_ch.clone());
            }
            _ => return Err(ArenaError::NotANode()),
        }
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

    fn bitwise_func(&self, idx: usize, stack_id: &mut Vec<usize>) -> bool {
        match_elem(
            self.get(idx),
            |n| {
                stack_id.append(&mut n.ch.clone());
                !"+.".contains(n.val.op)
            },
            |l| l.is_bitwise(),
        )
    }

    fn is_leaf(&self, idx: usize) -> bool {
        match_elem(self.get(idx), |_| false, |_| true)
    }

    fn is_cst(&self, idx: usize) -> bool {
        match_elem(self.get(idx), |_| false, |l| l.is_cst())
    }

    pub fn is_bitwise(&self, idx: usize) -> bool {
        let mut stack_id: Vec<usize> = vec![idx];
        while let Some(curr_idx) = stack_id.pop() {
            if !self.bitwise_func(curr_idx, &mut stack_id) {
                return false;
            }
        }

        true
    }

    // mba form: + a1.e1;a2.e2;...., where ai is an int, ei is a bitwise expression
    // a.e is referred to as mba_term
    // leaf are trivial case of mban -> rejected
    // therefore, if true, an mba necessarily has children -> update children
    pub fn is_mba<'a>(&'a self, idx: usize) -> bool {
        let mut mba_term_vec: Vec<usize> = Vec::new();
        let mut bitwise_vec: Vec<usize> = Vec::new();

        if self.is_leaf(idx) {
            return false;
        }

        if !fn_node(self.get(idx), |n| match n.val.op {
            '+' => {
                mba_term_vec.append(&mut n.ch.clone());
                true
            }
            _ => false,
        }) {
            return false;
        }

        // elem in stack_mba_term all have same '+' parent
        while let Some(curr_idx) = mba_term_vec.pop() {
            // if add operator -> not mba_term, parent not an mba
            if match_elem(self.get(curr_idx), |n| n.val.op == '+', |_| false) {
                return false;
            }

            // if leaf...
            if self.is_leaf(curr_idx) {
                continue;
            }

            // ...else: node with bitwise op, or '.' op
            // if '.' op, collect all children for bitwise check + check at most 1 node
            if !fn_node(self.get(curr_idx), |n| match n.val.op {
                '.' => {
                    let mut num_var_expr = 0;
                    n.ch.iter().all(|ch_idx| {
                        match_elem(self.get(*ch_idx), |_| bitwise_vec.push(*ch_idx), |_| ());
                        if !self.is_cst(*ch_idx) {
                            num_var_expr += 1;
                        }
                        num_var_expr < 2
                    })
                }
                _ => {
                    bitwise_vec.push(curr_idx);
                    true
                }
            }) {
                return false;
            }
        }

        while let Some(curr_idx) = bitwise_vec.pop() {
            if !self.bitwise_func(curr_idx, &mut bitwise_vec) {
                return false;
            }
        }

        true
    }
}

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
                let elem = arena.get_node(idx).expect("should have found node");

                // ... it has at least 2children, it should be merge/push to top of node_stack
                if let (true, Some(&mut par_idx)) = (elem.ch.len() > 1, node_stack.back_mut()) {
                    let (par_op, par_sign) =
                        arena.get_op_sign(par_idx).expect("should have found node");
                    // if same op,sign as top of stack -> merge children
                    if par_op == elem.val.op && par_sign == elem.val.sign {
                        arena.move_ch(par_idx, idx).expect("ch not moved");
                    } else {
                        // ... if not set as child
                        arena.push_ch(par_idx, idx).expect("ch not pushed");
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
                                arena.get_op_sign(idx).expect("should have found op.");
                            if curr_op == new_op && sign.len() == 0 {
                                continue;
                            }
                        }
                        if sign.len() > 0 {
                            sign_stack.push(mem::take(&mut sign));
                        }

                        take_node_stack(arena, &mut curr_node, &mut node_stack);

                        let expr = Expr::new(new_op, sign_stack.pop().unwrap_or("".to_string()));
                        curr_node = Some(arena.node(expr));
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
                    let val: ExprVal;
                    if let Ok(c_u) = elem.parse::<u32>() {
                        val = ExprVal::U32(c_u);
                    } else if let (true, Some(c_var)) = (elem.len() == 1, elem.chars().next()) {
                        val = ExprVal::Var(c_var);
                    } else {
                        return Err(ExprError::RPNSyntaxError());
                    }

                    let id_l = arena.leaf(Term {
                        val,
                        sign: mem::take(&mut sign),
                    });

                    match curr_node.as_mut() {
                        Some(idx) => arena
                            .push_ch(*idx, id_l)
                            .expect("should have been able to push."),
                        None => {
                            if rpn.len() == 0 {
                                return Ok(id_l);
                            }
                            return Err(ExprError::RPNSyntaxError());
                        }
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
