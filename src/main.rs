use std::env;

// use crate::error::ParseError;
// use crate::parser::{parse_line, read_lines};

use Dobf::error::ParseError;
use Dobf::parser::{parse_line, read_lines};

//mod error;
//mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(&args[1]) {
        // Consumes the iterator, returns an (Optional) String
        for line_res in lines {
            if let Ok(line) = line_res {
                match parse_line(line) {
                    Ok(cl_line) => println!("cleaned line: {:?}", cl_line),
                    Err(e) => println!("error cleaning {:?}", e),
                }
            }
        }
    }
}
