use std::env;

// use crate::error::ParseError;
// use crate::parser::{parse_line, read_lines};

use dobf::arena::{Arena, DAGFactory};
//use dobf::factory::DAGFactory;

use dobf::parser::{parse_rpn, read_lines};

//mod error;
//mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let test = 'x';
    println!("{}", format!("test {:#?}", test));

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(&args[1]) {
        // Consumes the iterator, returns an (Optional) String
        for line_res in lines {
            if let Ok(line) = line_res {
                /*match parse_rpn(line) {
                    Ok(cl_line) => {
                        println!("cleaned line: {:?}", cl_line);
                        let dag = DAGFactory::new_dag(&mut cl_line.clone()).unwrap();
                        let is_mba = dag.is_mba();
                        println!("      -> is_mba: {:#?}", is_mba);
                    }
                    Err(e) => println!("error cleaning {:?}", e),
                }*/
                match parse_rpn(line) {
                    Ok(cl_line) => {
                        let mut arena = Arena::new();
                        println!("cleaned line: {:?}", cl_line);
                        let elem_idx =
                            DAGFactory::new_dag(&mut cl_line.clone(), &mut arena).unwrap();
                        println!("{}", arena.elem_str(elem_idx));
                    }
                    Err(e) => println!("error cleaning {:?}", e),
                }
            }
        }
    }
}
