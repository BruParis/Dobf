use std::mem;

use super::arena::Arena;
use super::utils::{is_bitwise, is_mba, is_mba_term};

pub(super) enum Expr {
    MBA(MBA),         // -> op: +     - ch vec (mba/bitwise terms)
    MBATerm(MBATerm), // -> op: .     - ch: (cst, bitwise)
    Bitwise(Bitwise), // -> op: bitw. - ch: vec
    Other(Other),     // -> op: any   - ch: vec of anything
}

impl Expr {
    pub(super) fn op(&self) -> char {
        match self {
            Expr::MBA(_) => '+',
            Expr::MBATerm(_) => '.',
            Expr::Bitwise(e) => e.op,
            Expr::Other(e) => e.op,
        }
    }
    pub(super) fn ch(&self) -> Vec<usize> {
        match self {
            Expr::MBA(e) => e.ch.clone(),
            Expr::MBATerm(e) => vec![e.bitw],
            Expr::Bitwise(e) => e.ch.clone(),
            Expr::Other(e) => e.ch.clone(),
        }
    }
    pub(super) fn take_ch(&mut self) -> Vec<usize> {
        match self {
            Expr::MBA(e) => mem::take(&mut e.ch),
            Expr::MBATerm(e) => vec![e.bitw],
            Expr::Bitwise(e) => mem::take(&mut e.ch),
            Expr::Other(e) => mem::take(&mut e.ch),
        }
    }

    pub(super) fn push_ch_mut(&mut self, arena: &Arena, ch_idx: &Vec<usize>) -> Option<Expr> {
        match self {
            Expr::MBA(e) => e.push_ch_mut(arena, ch_idx),
            Expr::MBATerm(e) => e.push_ch_mut(arena, ch_idx),
            Expr::Bitwise(e) => e.push_ch_mut(arena, ch_idx),
            Expr::Other(e) => e.push_ch_mut(arena, ch_idx),
        }
    }
}

pub struct MBA {
    ch: Vec<usize>,
}

impl MBA {
    fn push_ch_mut(&mut self, arena: &Arena, ch_idx: &Vec<usize>) -> Option<Expr> {
        self.ch.append(&mut ch_idx.clone());
        if is_mba(arena, '+', &self.ch) {
            None
        } else {
            Some(Expr::Other(Other {
                ch: mem::take(&mut self.ch),
                op: '+',
            }))
        }
    }
}

pub struct MBATerm {
    bitw: usize,
}

impl MBATerm {
    fn push_ch_mut(&mut self, _arena: &Arena, ch_idx: &Vec<usize>) -> Option<Expr> {
        let mut ch = vec![self.bitw];
        ch.append(&mut ch_idx.clone());
        Some(Expr::Other(Other { ch, op: '.' }))
    }
}

pub struct Bitwise {
    ch: Vec<usize>,
    op: char,
}

impl Bitwise {
    fn push_ch_mut(&mut self, arena: &Arena, ch_idx: &Vec<usize>) -> Option<Expr> {
        self.ch.append(&mut ch_idx.clone());
        if is_bitwise(arena, self.op, &self.ch) {
            None
        } else {
            Some(Expr::Other(Other {
                ch: mem::take(&mut self.ch),
                op: self.op,
            }))
        }
    }
}

pub struct Other {
    ch: Vec<usize>,
    op: char,
}

impl Other {
    pub(super) fn new(op: char) -> Self {
        Self { ch: Vec::new(), op }
    }

    fn push_ch_mut(&mut self, arena: &Arena, ch_idx: &Vec<usize>) -> Option<Expr> {
        self.ch.append(&mut ch_idx.clone());
        match self.op {
            '+' => {
                if is_mba(arena, '+', &self.ch) {
                    Some(Expr::MBA(MBA {
                        ch: mem::take(&mut self.ch),
                    }))
                } else {
                    None
                }
            }
            '.' => {
                if is_mba_term(arena, '.', &self.ch) {
                    Some(Expr::MBATerm(MBATerm {
                        bitw: mem::take(&mut self.ch[0]),
                    }))
                } else {
                    None
                }
            }
            op => {
                if is_bitwise(arena, op, &self.ch) {
                    Some(Expr::Bitwise(Bitwise {
                        ch: mem::take(&mut self.ch),
                        op,
                    }))
                } else {
                    None
                }
            }
        }
    }
}
