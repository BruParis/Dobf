use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::error::ParseError;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn parse_line(line: String) -> Result<String, ParseError> {
    clean_line(line)
}

pub fn clean_line(mut line: String) -> Result<String, ParseError> {
    println!("expr: {}", line);
    let mut par_count = 0;
    let mut neg_count = 0;
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
            '~' => {
                neg_count += 1;
            }
            '+' | '-' | '^' | '&' | '|' => {
                if neg_count > 0 {
                    return Err(ParseError::NegSignOp(format!(
                        "neg sign before operator: ~{}",
                        w[1]
                    )));
                }

                if "-+^&|".contains(w[0]) {
                    return Err(ParseError::SuccessiveOp(format!(
                        "Successive operators: {}{}",
                        w[0], w[1]
                    )));
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
                    let mut sub_vec = String::new();
                    if (neg_count % 2) == 1 {
                        sub_vec.push('~')
                    }
                    sub_vec.push(w[1]);
                    neg_count = 0;

                    if par_count > 0 {
                        curr_term.push_str(&sub_vec);
                    } else {
                        res_str.push_str(&sub_vec);
                    }
                } else if w[1] != ' ' {
                    return Err(ParseError::WrongChar("Wrong char".to_string()));
                }
            }
        }

        if par_count < 0 {
            return Err(ParseError::MissOpenPar("Missing (".to_string()));
        }
    }
    res_str.push_str(&curr_term);

    if par_count > 0 {
        return Err(ParseError::MissClosePar("Missing )".to_string()));
    }

    println!("parc_count: {}", par_count);
    Ok(res_str)
}
