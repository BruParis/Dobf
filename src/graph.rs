use std::fs;

use crate::expr::arena::Arena;

pub struct Graph;
impl Graph {
    pub fn write_graph(arena: &Arena) {
        let graph_str = arena.graph_str();

        fs::write("expr_graph.dot", graph_str).expect("Unable to write file");
    }
}
