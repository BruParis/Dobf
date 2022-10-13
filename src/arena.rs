use std::collections::VecDeque;
use std::mem;

use crate::error::{ArenaError, ExprError};
use crate::expr::VarTerm;

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

fn compute_sign(cst: u32, sign: String) -> u32 {
    match sign.as_str() {
        "" => cst,
        "-" => (-(cst as i32)) as u32,
        "~" => !cst,
        "~-" => !((-(cst as i32)) as u32),
        "-~" => (-((!cst) as i32)) as u32,
        _ => unreachable!(),
    }
}

fn is_mba(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if op != '+' {
        return false;
    }
    ch.iter().all(|&ch| {
        match_elem(
            arena.get(ch),
            |n| node_is_bitwise(&n.expr, n.cst) || node_is_mba_term(&n.expr, n.cst),
            |_| true,
        )
    })
}

fn node_is_mba_term(expr: &EExpr, cst: Option<u32>) -> bool {
    matches!(expr, EExpr::MBATerm(_)) && !cst.is_none()
}

fn node_is_bitwise(expr: &EExpr, cst: Option<u32>) -> bool {
    matches!(expr, EExpr::Bitwise(_)) && cst.is_none()
}

fn ch_bitwise(arena: &Arena, ch: &Vec<usize>) -> bool {
    ch.iter()
        .all(|&ch| match_elem(arena.get(ch), |n| node_is_bitwise(&n.expr, n.cst), |_| true))
}

fn is_bitwise(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if !"^&|".contains(op) {
        return false;
    }
    ch_bitwise(arena, ch)
}

fn is_mba_term(arena: &Arena, op: char, ch: &Vec<usize>) -> bool {
    if op != '.' {
        return false;
    }

    if ch.len() != 1 {
        return false;
    }

    ch_bitwise(arena, ch)
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

pub fn fn_node_mut<F, R>(elem: &mut Elem, func: &mut F) -> R
where
    F: FnMut(&mut Node) -> R,
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
            |n| (n.idx, n.sign.clone()),
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

pub enum EExpr {
    MBA(MBA),         // -> op: +     - ch vec (mba/bitwise terms)
    MBATerm(MBATerm), // -> op: .     - ch: (cst, bitwise)
    Bitwise(Bitwise), // -> op: bitw. - ch: vec
    Other(Other),     // -> op: any   - ch: vec of anything
}

pub struct MBA {
    ch: Vec<usize>,
}

pub struct MBATerm {
    bitw: usize,
}

pub struct Bitwise {
    ch: Vec<usize>,
    op: char,
}

pub struct Other {
    ch: Vec<usize>,
    op: char,
}

impl EExpr {
    fn op(&self) -> char {
        match self {
            EExpr::MBA(_) => '+',
            EExpr::MBATerm(_) => '.',
            EExpr::Bitwise(e) => e.op,
            EExpr::Other(e) => e.op,
        }
    }

    fn ch(&self) -> Vec<usize> {
        match self {
            EExpr::MBA(e) => e.ch.clone(),
            EExpr::MBATerm(e) => vec![e.bitw],
            EExpr::Bitwise(e) => e.ch.clone(),
            EExpr::Other(e) => e.ch.clone(),
        }
    }

    fn take_ch(&mut self) -> Vec<usize> {
        match self {
            EExpr::MBA(e) => mem::take(&mut e.ch),
            EExpr::MBATerm(e) => vec![e.bitw],
            EExpr::Bitwise(e) => mem::take(&mut e.ch),
            EExpr::Other(e) => mem::take(&mut e.ch),
        }
    }

    fn compute(&self, cst1: u32, cst2: u32) -> u32 {
        match self.op() {
            '+' => cst1 + cst2,
            '.' => cst1 * cst2,
            '^' => cst1 ^ cst2,
            '|' => cst1 | cst2,
            '&' => cst1 & cst2,
            _ => unreachable!(),
        }
    }
}

pub struct Node {
    pub idx: usize,
    sign: String,
    expr: EExpr,
    cst: Option<u32>,
    par: Option<usize>,
}

impl Node {
    fn new(idx: usize, sign: String, op: char) -> Self {
        Node {
            idx,
            sign,
            expr: EExpr::Other(Other { ch: Vec::new(), op }),
            cst: None,
            par: None,
        }
    }

    fn push_cst(&mut self, cst: u32) {
        if let Some(old_cst) = self.cst.take() {
            self.cst = Some(self.expr.compute(old_cst, cst));
        } else {
            self.cst = Some(cst);
        }
    }

    fn graph_label_str(&self) -> String {
        let main_str = format!("{} [label=<{}{}>]\n", self.idx, self.sign, self.expr.op());

        if let Some(cst) = self.cst {
            let cst_node = format!("\"{}c\" [label=<{}>]\n", self.idx, cst);
            format!("{}{}", main_str, cst_node)
        } else {
            main_str
        }
    }

    fn graph_edge_str(&self) -> String {
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

    fn op(&self) -> char {
        self.expr.op()
    }

    fn ch(&self) -> Vec<usize> {
        self.expr.ch()
    }

    fn pref_suff(&self) -> (String, String) {
        let (mut p_str, mut s_str) = (String::new(), String::new());
        let closed_par = self.sign.len() > 0;
        if self.sign.len() > 0 {
            p_str = format!("{}({}{}", &self.sign, &self.op(), p_str);
        //*p_str = &self.sign + "(" + &self.op.to_string() + p_str;
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
        match self.get(idx) {
            Elem::Node(n) => n.ch().clone(),
            _ => vec![],
        }
    }

    pub fn get_ch_len(&self, idx: usize) -> usize {
        match self.get(idx) {
            Elem::Node(n) => n.ch().len(),
            _ => 0,
        }
    }

    pub fn get_num_terms(&self, idx: usize) -> usize {
        match self.get(idx) {
            Elem::Node(n) => {
                if n.cst.is_none() {
                    n.ch().len()
                } else {
                    n.ch().len() + 1
                }
            }
            _ => 0,
        }
    }

    fn copy_op_sign(&self, idx: usize) -> Option<(char, String)> {
        match self.get(idx) {
            Elem::Node(n) => Some((n.op(), n.sign.clone())),
            _ => None,
        }
    }

    fn take_cst(&mut self, idx: usize) -> Option<u32> {
        match self.get_mut(idx) {
            Elem::Node(n) => n.cst.take(),
            _ => None,
        }
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
    fn push_ch_expr(&mut self, expr: &mut EExpr, ch_idx: &Vec<usize>) {
        if let Some(new_expr) = match expr {
            EExpr::MBA(e) => {
                e.ch.append(&mut ch_idx.clone());
                if is_mba(self, '+', &e.ch) {
                    None
                } else {
                    Some(EExpr::Other(Other {
                        ch: mem::take(&mut e.ch),
                        op: '+',
                    }))
                }
            }
            EExpr::MBATerm(e) => {
                let mut ch = vec![e.bitw];
                ch.append(&mut ch_idx.clone());
                Some(EExpr::Other(Other { ch, op: '.' }))
            }
            EExpr::Bitwise(e) => {
                e.ch.append(&mut ch_idx.clone());
                if is_bitwise(self, e.op, &e.ch) {
                    None
                } else {
                    Some(EExpr::Other(Other {
                        ch: mem::take(&mut e.ch),
                        op: e.op,
                    }))
                }
            }
            EExpr::Other(e) => {
                e.ch.append(&mut ch_idx.clone());
                match e.op {
                    '+' => {
                        if is_mba(self, '+', &e.ch) {
                            Some(EExpr::MBA(MBA {
                                ch: mem::take(&mut e.ch),
                            }))
                        } else {
                            None
                        }
                    }
                    '.' => {
                        if is_mba_term(self, '.', &e.ch) {
                            Some(EExpr::MBATerm(MBATerm {
                                bitw: mem::take(&mut e.ch[0]),
                            }))
                        } else {
                            None
                        }
                    }
                    c => {
                        if is_bitwise(self, c, &e.ch) {
                            Some(EExpr::Bitwise(Bitwise {
                                ch: mem::take(&mut e.ch),
                                op: c,
                            }))
                        } else {
                            None
                        }
                    }
                }
            }
        } {
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
                        Some(node.expr.compute(acc_v, cst))
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

    fn push_ch(&mut self, idx: usize, idx_ch_vec: &Vec<usize>) -> Result<(), ArenaError> {
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

    fn push_cst(&mut self, idx: usize, cst: u32) -> Result<(), ArenaError> {
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
    fn move_ch(&mut self, idx_to: usize, idx_from: usize) -> Result<(), ArenaError> {
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
        match_elem(
            self.get(idx),
            |n| matches!(n.expr, EExpr::MBA(_)),
            |_| false,
        )
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
