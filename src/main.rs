use itertools::Itertools;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// use crate::error::error::error::ParseError;
mod error;

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

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_line(mut line: String) -> Result<String, error::ParseError> {
    println!("expr: {}", line);
    let mut par_count = 0;
    let mut res_str = String::new();
    let mut curr_term = String::new();
    let mut curr_term_has_op = false;

    // remove whitespace
    line = line.replace(" ", "");
    line = " ".to_string() + &line;

    let inter = line.chars().collect::<Vec<char>>();
    for w in inter.windows(2) {
        // println!(
        //     "{:};{:} - curr: {:} - res: {:}",
        //     w[0], w[1], curr_term, res_str
        // );
        match w[1] {
            '(' => {
                par_count += 1;
            }
            ')' => {
                par_count -= 1;
                if curr_term_has_op && w[0] != ')' {
                    curr_term.push(')');
                    curr_term = "(".to_string() + &curr_term;
                }
                if par_count == 0 {
                    res_str.push_str(&curr_term);
                    curr_term = String::new();
                }

                curr_term_has_op = false;
            }
            '+' | '-' | '^' | '&' | '|' => {
                if "+^&|".contains(w[0]) {
                    return Err(error::ParseError::SuccessiveOp(
                        "Successive operators: {prev}{next}".to_string(),
                    ));
                }
                if par_count > 0 {
                    curr_term.push(w[1]);
                    curr_term_has_op = true;
                } else {
                    res_str.push(w[1]);
                }
            }
            _ => {
                if w[1].is_alphanumeric() {
                    if par_count > 0 {
                        curr_term.push(w[1]);
                    } else {
                        res_str.push(w[1]);
                    }
                } else if w[1] != ' ' {
                    return Err(error::ParseError::WrongChar("Wrong char".to_string()));
                }
            }
        }

        if par_count < 0 {
            return Err(error::ParseError::MissOpenPar("Missing (".to_string()));
        }
    }
    res_str.push_str(&curr_term);

    if par_count > 0 {
        return Err(error::ParseError::MissClosePar("Missing )".to_string()));
    }

    println!("parc_count: {}", par_count);
    Ok(res_str)
}
