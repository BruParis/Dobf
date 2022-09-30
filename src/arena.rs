use std::collections::VecDeque;
use std::mem;

use crate::error::{ArenaError, ExprError};
use crate::expr::{Expr, ExprVal, Term};

pub struct Arena {
    elems: Vec<Elem>,
    free_slots: Vec<usize>,
}

pub enum Elem {
    Node(Node),
    Leaf(Leaf),
    Free,
}

impl Elem {
    fn get_par_id(&self) -> Option<usize> {
        match self {
            Elem::Node(n) => n.par,
            Elem::Leaf(l) => l.par,
            _ => return panic!("idx -> free elem"),
        }
    }

    fn get_idx_sign(&self) -> (usize, String) {
        match self {
            Elem::Node(n) => (n.idx, n.val.sign.clone()),
            Elem::Leaf(l) => (l.idx, l.val.sign.clone()),
            _ => return panic!("idx -> free elem"),
        }
    }

    fn graph_label_str(&self) -> String {
        match self {
            Elem::Node(n) => n.graph_label_str(),
            Elem::Leaf(l) => l.graph_label_str(),
            _ => panic!("idx -> free elem"),
        }
    }
    fn graph_edge_str(&self) -> Option<String> {
        match self {
            Elem::Node(n) => Some(n.graph_edge_str()),
            Elem::Leaf(l) => None,
            _ => panic!("idx -> free elem"),
        }
    }
}

pub struct Node {
    pub idx: usize,
    val: Expr,
    ch: Vec<usize>,
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
    val: Term,
    par: Option<usize>,
}

impl Leaf {
    fn graph_label_str(&self) -> String {
        format!(
            "{} [label=<{}{:?}>]\n",
            self.idx, self.val.sign, self.val.val
        )
    }
}

impl Arena {
    pub fn new() -> Self {
        Arena {
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

    fn get(&self, idx: usize) -> Result<&Elem, ArenaError> {
        self.elems.get(idx).ok_or(ArenaError::NotFound())
    }

    fn get_node(&self, idx: usize) -> Result<&Node, ArenaError> {
        let e = self.elems.get(idx).ok_or(ArenaError::NotFound())?;

        match e {
            Elem::Node(n) => Ok(n),
            _ => Err(ArenaError::NotANode()),
        }
    }

    fn get_op_sign(&self, idx: usize) -> Result<(char, String), ArenaError> {
        self.get(idx).and_then(|e| match e {
            Elem::Node(n) => Ok((n.val.op, n.val.sign.clone())),
            _ => Err(ArenaError::ElemIsLeaf()),
        })
    }

    fn get_mut(&mut self, idx: usize) -> Result<&mut Elem, ArenaError> {
        self.elems.get_mut(idx).ok_or(ArenaError::NotFound())
    }

    fn remove_elem(&mut self, idx: usize) -> Result<Elem, ArenaError> {
        let rem_elem = mem::replace(self.get_mut(idx)?, Elem::Free);
        self.free_slots.push(idx);
        Ok(rem_elem)
    }

    fn get_preorder(&self, idx: usize) -> Vec<usize> {
        let mut idx_stack = vec![idx];
        let mut res: Vec<usize> = Vec::new();
        while let Some(idx) = idx_stack.pop() {
            res.push(idx);
            let e = self.get(idx).expect("should have found elem");
            match e {
                Elem::Node(n) => idx_stack.append(&mut n.ch.clone()),
                Elem::Free => return panic!("idx -> free elem"),
                _ => (),
            }
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

    pub fn elem_str(&self, idx: usize) -> String {
        let mut str_stack: VecDeque<String> = VecDeque::new();
        let mut suff_stack: Vec<String> = Vec::new();
        let mut par_idx: Vec<usize> = Vec::new();
        let mut res = String::new();

        let mut pre_order = self.get_preorder(idx);
        pre_order.reverse();

        while let Some(idx) = pre_order.pop() {
            let e = self.get(idx).expect("should have found elem");

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

            match e {
                Elem::Node(n) => {
                    let mut pre_str = String::new();
                    let mut suff_str = String::new();
                    n.val.pref_suff(&mut pre_str, &mut suff_str);
                    res.push_str(&pre_str);
                    suff_stack.push(suff_str);

                    par_idx.push(idx);
                }
                Elem::Leaf(l) => {
                    res.push_str(&format!("{:?}", l.val));
                }
                _ => return panic!("idx -> free elem"),
            }
        }

        res.push_str(&suff_stack.join(""));

        while let Some(r_str) = str_stack.pop_front() {
            res.push_str(&r_str);
        }

        res
    }

    fn set_par(&mut self, idx: usize, par_id: usize) -> Result<(), ArenaError> {
        match self.get_mut(idx)? {
            Elem::Node(n) => n.par = Some(par_id),
            Elem::Leaf(l) => l.par = Some(par_id),
            _ => return Err(ArenaError::ElemIsFree()),
        }

        Ok(())
    }

    fn push_ch(&mut self, idx: usize, idx_ch: usize) -> Result<(), ArenaError> {
        self.get(idx_ch)?;
        let mut par_idx_opt = None;
        match self.get_mut(idx)? {
            Elem::Node(n) => {
                par_idx_opt = Some(n.idx);
                n.ch.push(idx_ch);
            }
            Elem::Leaf(_) => return Err(ArenaError::ParentIsLeaf()),
            _ => return Err(ArenaError::ElemIsFree()),
        }
        self.set_par(idx_ch, par_idx_opt.expect("should have found parent id"))?;
        Ok(())
    }

    fn push_ch_vec(&mut self, idx: usize, idx_ch_vec: &mut Vec<usize>) -> Result<(), ArenaError> {
        idx_ch_vec
            .iter()
            .try_for_each(|idx| self.get(*idx).and_then(|_| Ok(())));
        let mut par_idx_opt = None;
        match self.get_mut(idx)? {
            Elem::Node(n) => {
                par_idx_opt = Some(n.idx);
                n.ch.append(idx_ch_vec);
            }
            Elem::Leaf(_) => return Err(ArenaError::ParentIsLeaf()),
            _ => return Err(ArenaError::ElemIsFree()),
        }

        for idx_ch in idx_ch_vec {
            self.set_par(*idx_ch, par_idx_opt.expect("should have found parent id"))?;
        }
        Ok(())
    }

    fn move_ch(&mut self, idx_to: usize, idx_from: usize) -> Result<(), ArenaError> {
        let mut from_ch: Vec<usize>;
        match self.get(idx_from)? {
            Elem::Node(from) => from_ch = from.ch.clone(),
            _ => return Err(ArenaError::NotANode()),
        }

        match self.get_mut(idx_to)? {
            Elem::Node(to) => {
                to.ch.append(&mut from_ch.clone());
            }
            _ => return Err(ArenaError::NotANode()),
        }
        // update children's new parent id
        for ch_idx in from_ch {
            match self.get_mut(ch_idx)? {
                Elem::Node(n_ch) => n_ch.par = Some(idx_to),
                Elem::Leaf(l_ch) => l_ch.par = Some(idx_to),
                _ => return Err(ArenaError::ElemIsFree()),
            }
        }

        self.remove_elem(idx_from)?;

        Ok(())
    }
}

pub struct DAGFactory;
impl DAGFactory {
    pub fn new_dag(rpn: &mut VecDeque<String>, arena: &mut Arena) -> Result<usize, ExprError> {
        let expr_id = DAGFactory::build_expr(rpn, arena)?;

        Ok(expr_id)
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
