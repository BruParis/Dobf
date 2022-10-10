use crate::arena::{fn_node, match_elem, Arena};
use crate::expr::Term;

fn reduce(arena: &mut Arena, root_idx: usize) {
    let mut node_idx_vec = arena.get_preorder(root_idx, false);

    while let Some(idx) = node_idx_vec.pop() {
        let mut ch_vec = arena.get_ch(idx);
        if !arena.is_mba(idx) {
            continue;
        }

        let p_str = arena.elem_str(idx);
        print!("mba: {}", p_str);

        for ch in ch_vec {
            fn_node(arena.get(idx), |n| match n.val.op {
                '.' => {
                    let mut cst_t: Option<Term> = None;
                    let mut bitw_idx: Option<usize> = None;

                    n.ch.iter().map(|ch| {
                        match_elem(
                            arena.get(*ch),
                            |_| bitw_idx = Some(*ch),
                            |l_ch| cst_t = Some(l_ch.val),
                        )
                    });

                    false
                }
                _ => true,
            });
        }
    }
}
